use crate::{hex_fmt::HexFmt, http400_unless, http500_unless};
use cgi::{Request, Response, text_response};
use sqlite::Connection;
use std::collections::HashMap;

pub(crate) fn handler(db: Connection, req: Request) -> Response {
    let form = http400_unless!("Invalid form charset", String::from_utf8(req.into_body()));
    let mut formdata = HashMap::<&str, &str>::new();

    if !form.is_empty() {
        for field in form.lines() {
            match field.split_once("=") {
                None => return text_response(400, "No '=' in form field.\r\n"),
                Some((k, v)) => {
                    if !v.is_empty() {
                        formdata.insert(k, v);
                    }
                }
            }
        }
    }

    http500_unless!(
        "Failed to UPDATE person",
        db.execute(format!(
            "UPDATE person SET url = {} WHERE id = unhex('{}')",
            match formdata.remove("url") {
                None => "NULL".to_string(),
                Some(url) => format!("CAST(unhex('{}') AS TEXT)", HexFmt::new(url.as_bytes())),
            },
            HexFmt::new(formdata.remove("id").unwrap_or("").as_bytes())
        ))
    );

    text_response(200, "Success.\r\n")
}
