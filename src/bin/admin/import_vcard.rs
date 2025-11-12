use crate::{hex_fmt::HexFmt, http500_unless, util::UploadedFile};
use cgi::{Response, text_response};
use ical::{VcardParser, parser::Component};
use sqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn handler(db: Connection, file: UploadedFile) -> Response {
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
                            "INSERT OR IGNORE INTO person(name) VALUES (unhex('{}'))",
                            hex_name
                        ))
                    );

                    http500_unless!(
                        "Failed to INSERT INTO pzk",
                        db.execute(format!("INSERT INTO pzk VALUES ({}, (SELECT id FROM person WHERE name = unhex('{}')))", snapshot_time, hex_name))
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
