use crate::{http400_unless, http500_unless, nullint_fmt::NullIntFmt};
use cgi::{Request, Response, text_response};
use sqlite::Connection;
use std::collections::HashMap;

pub(crate) fn handler(db: Connection, req: Request) -> Response {
    let form = http400_unless!("Invalid form charset", String::from_utf8(req.into_body()));
    let mut formdata = HashMap::<&str, i64>::new();

    if !form.is_empty() {
        for field in form.split("&") {
            match field.split_once("=") {
                None => return text_response(400, "No '=' in form field.\r\n"),
                Some((k, v)) => {
                    if !v.is_empty() {
                        formdata.insert(k, http400_unless!("Invalid int value", v.parse::<i64>()));
                    }
                }
            }
        }
    }

    http500_unless!(
        "Failed to UPDATE person",
        db.execute(format!(
            "UPDATE person SET birth_year = {}, birth_month = {}, birth_day = {} WHERE id = {}",
            NullIntFmt::new(formdata.remove("year"), "NULL"),
            NullIntFmt::new(formdata.remove("month"), "NULL"),
            NullIntFmt::new(formdata.remove("day"), "NULL"),
            NullIntFmt::new(formdata.remove("id"), "NULL")
        ))
    );

    text_response(200, "Success.\r\n")
}
