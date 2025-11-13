use crate::{http500_unless, nullint_fmt::NullIntFmt};
use cgi::{Request, Response, html_response, text_response};
use html::{root::Html, tables::Table};
use sqlite::Connection;

pub(crate) fn handler(db: Connection, _: Request) -> Response {
    macro_rules! read_col {
        ($row:expr, $col:expr, $typ:ty) => {
            http500_unless!(
                format!("Failed to read column {}", $col),
                $row.try_read::<$typ, _>($col)
            )
        };
    }

    let mut table = Table::builder();

    table.table_row(|tr| {
        tr.table_header(|th| th.text("Name"))
            .table_header(|th| th.text("Birthday"))
    });

    let query = http500_unless!(
        "Failed to prepare query",
        db.prepare("SELECT * FROM person WHERE id IN (SELECT person_id FROM pzk WHERE snapshot_time = (SELECT max(snapshot_time) FROM pzk)) ORDER BY id DESC")
    );

    for i in query {
        let row = http500_unless!("Failed to fetch row", i);
        let name = read_col!(row, "name", &str).to_string();
        let birth_year = read_col!(row, "birth_year", Option<i64>);
        let birth_month = read_col!(row, "birth_month", Option<i64>);
        let birth_day = read_col!(row, "birth_day", Option<i64>);

        table.table_row(|tr| {
            tr.table_cell(|td| td.text(name)).table_cell(|td| {
                td.text(format!(
                    "{}-{}-{}",
                    NullIntFmt::new(birth_year, "?"),
                    NullIntFmt::new(birth_month, "?"),
                    NullIntFmt::new(birth_day, "?")
                ))
            })
        });
    }

    html_response(
        200,
        Html::builder()
            .body(|body| {
                body.table(|tbl| {
                    *tbl = table;
                    tbl
                })
            })
            .build()
            .to_string(),
    )
}
