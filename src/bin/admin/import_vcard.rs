use crate::{hex_fmt::HexFmt, http500_unless, nullint_fmt::NullIntFmt, util::UploadedFile};
use cgi::{Response, text_response};
use ical::{VcardParser, parser::Component};
use regex_lite::{Captures, Regex};
use sqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn handler(db: Connection, file: UploadedFile) -> Response {
    let yyyymmdd = http500_unless!(
        "Failed to compile regexp",
        Regex::new(r"\A(--|[0-9]{4})([0-9]{2})([0-9]{2})\z")
    );

    http500_unless!(
        "Failed to init database",
        db.execute(include_str!("schema.sql"))
    );

    http500_unless!("Failed to init transaction", db.execute("BEGIN IMMEDIATE"));

    let snapshot_time = http500_unless!(
        "Failed to travel in time",
        SystemTime::now().duration_since(UNIX_EPOCH)
    )
    .as_secs();

    for i in VcardParser::new(file) {
        match i {
            Err(err) => return text_response(400, format!("Invalid vCard: {}\r\n", err)),
            Ok(mut vcard) => match contact_prop(&mut vcard, "FN") {
                None => {}
                Some(name) => {
                    let hex_name = HexFmt::new(name.as_bytes());

                    http500_unless!(
                        "Failed to INSERT INTO person",
                        db.execute(format!(
                            "INSERT OR IGNORE INTO person(name) VALUES (CAST(unhex('{}') AS TEXT))",
                            hex_name
                        ))
                    );

                    match contact_prop(&mut vcard, "BDAY") {
                        None => {}
                        Some(birthday) => match yyyymmdd.captures(birthday.as_str()) {
                            None => {
                                return text_response(
                                    501,
                                    format!("Birthday is not like this: {}\r\n", yyyymmdd),
                                );
                            }
                            Some(cap) => {
                                http500_unless!(
                                    "Failed to UPDATE person",
                                    db.execute(format!(
                                        "UPDATE person SET birth_year = {}, birth_month = {}, birth_day = {} WHERE name = CAST(unhex('{}') AS TEXT)",
                                        parse(&cap, 1), parse(&cap, 2), parse(&cap, 3), hex_name
                                    ))
                                );
                            }
                        },
                    }

                    http500_unless!(
                        "Failed to INSERT INTO pzk",
                        db.execute(format!(
                            "INSERT OR IGNORE INTO pzk VALUES ({}, (SELECT id FROM person WHERE name = CAST(unhex('{}') AS TEXT)))",
                            snapshot_time, hex_name
                        ))
                    );
                }
            },
        }
    }

    http500_unless!("Failed to commit transaction", db.execute("COMMIT"));

    return text_response(200, "Success.\r\n");
}

fn contact_prop(contact: &mut impl Component, prop: &'static str) -> Option<String> {
    match contact.get_property_mut(prop) {
        None => None,
        Some(val) => val.value.take(),
    }
}

fn parse<'a>(cap: &Captures<'a>, i: usize) -> NullIntFmt {
    NullIntFmt::new(
        match cap.get(i) {
            None => None,
            Some(m) => match m.as_str().parse::<i64>() {
                Err(_) => None,
                Ok(x) => Some(x),
            },
        },
        "NULL",
    )
}
