use cgi::{
    Request, Response, binary_response, handle, http::header::CONTENT_DISPOSITION, text_response,
};
use sqlite::{Connection, OpenFlags};
use std::{env::var_os, io::Write};

fn main() {
    handle(handler);
}

fn handler(_: Request) -> Response {
    macro_rules! http500_unless {
        ($errmsg:expr, $result:expr) => {
            match $result {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("{}: {}", $errmsg, err);
                    return text_response(500, "Error\r\n");
                }
            }
        };
    }

    macro_rules! read_col {
        ($row:expr, $col:expr, $typ:ty) => {
            http500_unless!(
                format!("Failed to read column {}", $col),
                $row.try_read::<$typ, _>($col)
            )
        };
    }

    const PZKMGR_DB: &str = "PZKMGR_DB";

    let mut db = match var_os(PZKMGR_DB) {
        None => {
            eprintln!("Env var \"{}\" missing.", PZKMGR_DB);
            return text_response(500, "Error\r\n");
        }
        Some(path) => http500_unless!(
            "Failed to open database",
            Connection::open_with_flags(path, OpenFlags::new().with_read_only())
        ),
    };

    http500_unless!(
        "Failed to set database lock timeout",
        db.set_busy_timeout(4096)
    );

    let query = http500_unless!(
        "Failed to prepare query",
        db.prepare("SELECT snapshot_time, COUNT(*) AS people FROM pzk GROUP BY snapshot_time ORDER BY snapshot_time")
    );

    let mut body = Vec::<u8>::new();

    writeln!(&mut body, "snapshot_time,people").unwrap();

    for i in query {
        let row = http500_unless!("Failed to fetch row", i);

        writeln!(
            &mut body,
            "{},{}",
            read_col!(row, "snapshot_time", i64),
            read_col!(row, "people", i64)
        )
        .unwrap();
    }

    let mut resp = binary_response(200, "text/csv; charset=utf-8", body);

    resp.headers_mut().insert(
        CONTENT_DISPOSITION,
        "attachment; filename=\"pzkmgr-chart.csv\"".parse().unwrap(),
    );

    resp
}
