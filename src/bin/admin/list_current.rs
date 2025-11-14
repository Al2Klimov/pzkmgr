use crate::http500_unless;
use cgi::{Request, Response, html_response, text_response};
use html::{forms::builders::FormBuilder, root::Html, tables::Table};
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
            .table_header(|th| th.text("URL"))
    });

    let query = http500_unless!(
        "Failed to prepare query",
        db.prepare("SELECT * FROM person WHERE id IN (SELECT person_id FROM pzk WHERE snapshot_time = (SELECT max(snapshot_time) FROM pzk)) ORDER BY id DESC")
    );

    for i in query {
        let row = http500_unless!("Failed to fetch row", i);
        let id = read_col!(row, "id", i64).to_string();
        let name = read_col!(row, "name", &str).to_string();
        let url = read_col!(row, "url", Option<&str>).map(|url| url.to_string());
        let birth_year = read_col!(row, "birth_year", Option<i64>);
        let birth_month = read_col!(row, "birth_month", Option<i64>);
        let birth_day = read_col!(row, "birth_day", Option<i64>);

        table.table_row(|tr| {
            tr.table_cell(|td| td.text(name))
                .table_cell(|td| {
                    td.form(|form| {
                        form.target("_blank")
                            .action("?change-birthday")
                            .method("POST")
                            .input(|input| input.name("id").type_("hidden").value(id.clone()));

                        number_input(form, "year", "1000", "9999", birth_year);
                        number_input(form, "month", "1", "12", birth_month);
                        number_input(form, "day", "1", "31", birth_day);

                        form.input(|input| input.type_("submit").value("Save"))
                    })
                })
                .table_cell(|td| {
                    td.form(|form| {
                        form.target("_blank")
                            .action("?change-url")
                            .method("POST")
                            .enctype("text/plain")
                            .input(|input| input.name("id").type_("hidden").value(id))
                            .input(|input| {
                                input.name("url").type_("text");

                                match url {
                                    None => {}
                                    Some(uri) => {
                                        input.value(uri);
                                    }
                                }

                                input
                            })
                            .input(|input| input.type_("submit").value("Save"))
                    })
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

fn number_input(
    form: &mut FormBuilder,
    name: &'static str,
    min: &'static str,
    max: &'static str,
    value: Option<i64>,
) {
    form.input(|input| {
        input.name(name).type_("number").min(min).max(max);

        match value {
            None => {}
            Some(v) => {
                input.value(v.to_string());
            }
        }

        input
    });
}
