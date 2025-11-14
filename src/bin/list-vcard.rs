use cgi::{
    Request, Response, binary_response, handle, http::header::CONTENT_DISPOSITION, text_response,
};
use ical::{
    generator::Emitter,
    parser::{Component, vcard::component::VcardContact},
    property::Property,
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
        db.prepare("SELECT * FROM person WHERE id IN (SELECT person_id FROM pzk WHERE snapshot_time = (SELECT max(snapshot_time) FROM pzk))")
    );

    let mut body = Vec::<u8>::new();

    for i in query {
        let row = http500_unless!("Failed to fetch row", i);
        let mut vc = VcardContact::new();

        let name = read_col!(row, "name", &str);
        let url = read_col!(row, "url", Option<&str>);
        let birth_year = read_col!(row, "birth_year", Option<i64>);
        let birth_month = read_col!(row, "birth_month", Option<i64>);
        let birth_day = read_col!(row, "birth_day", Option<i64>);

        vc.add_property(Property {
            name: "VERSION".to_string(),
            params: None,
            value: Some("3.0".to_string()),
        });

        vc.add_property(Property {
            name: "FN".to_string(),
            params: None,
            value: Some(name.to_string()),
        });

        match (birth_month, birth_day) {
            (Some(month), Some(day)) => {
                vc.add_property(Property {
                    name: "BDAY".to_string(),
                    params: Some(vec![("VALUE".to_string(), vec!["DATE".to_string()])]),
                    value: Some(match birth_year {
                        None => format!("--{:0>2}{:0>2}", month, day),
                        Some(year) => format!("{:0>4}{:0>2}{:0>2}", year, month, day),
                    }),
                });
            }
            _ => {}
        }

        match url {
            None => {}
            Some(uri) => {
                vc.add_property(Property {
                    name: "URL".to_string(),
                    params: None,
                    value: Some(uri.to_string()),
                });
            }
        }

        body.append(vc.generate().into_bytes().by_ref());
    }

    let mut resp = binary_response(200, "text/vcard; charset=utf-8", body);

    resp.headers_mut().insert(
        CONTENT_DISPOSITION,
        "attachment; filename=\"pzkmgr.vcf\"".parse().unwrap(),
    );

    resp
}
