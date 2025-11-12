use crate::{hex_fmt::HexFmt, http500_unless, util::UploadedFile};
use cgi::{Response, text_response};
use csv::ReaderBuilder;
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

    for i in ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(file)
        .records()
    {
        match i {
            Err(err) => return text_response(400, format!("Invalid CSV: {}\r\n", err)),
            Ok(row) => match row.get(0) {
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
                        // Duplicate name in input => same person_id twice => INSERT OR IGNORE
                        db.execute(format!("INSERT OR IGNORE INTO pzk VALUES ({}, (SELECT id FROM person WHERE name = unhex('{}')))", snapshot_time, hex_name))
                    );
                }
            },
        }
    }

    http500_unless!("Failed to commit transaction", db.execute("COMMIT"));

    return text_response(200, "Success.\r\n");
}
