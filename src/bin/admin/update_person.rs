use crate::{
    hex_fmt::HexFmt, http400_unless, http500_unless, nullint_fmt::NullIntFmt, util::parse_nullint,
};
use cgi::{Request, Response, text_response};
use regex_lite::Regex;
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

    let mut dd = NullIntFmt::new(None, None, "NULL");
    let mut mm = dd.clone();
    let mut yyyy = dd.clone();

    match formdata.remove("birthday") {
        None => {}
        Some(birthday) => {
            let ddmmyyyy =
                Regex::new(r"\A([0-9]{1,2}|-*)\.([0-9]{1,2}|-*)\.([0-9]{4}|-*)\z").unwrap();

            match ddmmyyyy.captures(birthday) {
                None => {
                    return text_response(
                        400,
                        format!("Birthday is not like this: {}\r\n", ddmmyyyy),
                    );
                }
                Some(caps) => {
                    dd = parse_nullint(&caps, 1);
                    mm = parse_nullint(&caps, 2);
                    yyyy = parse_nullint(&caps, 3);
                }
            }
        }
    }

    http500_unless!(
        "Failed to UPDATE person",
        db.execute(format!(
            "UPDATE person SET birth_day = {}, birth_month = {}, birth_year = {}, url = {} WHERE id = CAST(unhex('{}') AS INTEGER)",
            dd, mm, yyyy,
            match formdata.remove("url") {
                None => "NULL".to_string(),
                Some(url) => format!("CAST(unhex('{}') AS TEXT)", HexFmt::new(url.as_bytes())),
            },
            HexFmt::new(formdata.remove("id").unwrap_or("").as_bytes())
        ))
    );

    if db.change_count() < 1 {
        return text_response(404, "No such entry.\r\n");
    }

    text_response(200, "Success.\r\n")
}
