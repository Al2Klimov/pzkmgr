use crate::{hex_fmt::HexFmt, http500_unless};
use cgi::{Request, Response, http::header, text_response};
use csv::ReaderBuilder;
use multipart::server::Multipart;
use sqlite::Connection;
use std::{io::Cursor, time};

pub(crate) fn handler(db: Connection, req: Request) -> Response {
    const BOUNDARY: &str = "boundary=";

    let oboundary = match req.headers().get("X-CGI-Content-Type") {
        None => None,
        Some(ct) => match ct.to_str() {
            Err(_) => None,
            Ok(cts) => match cts.find(BOUNDARY) {
                None => None,
                Some(pos) => {
                    let mut sub = &cts[pos + BOUNDARY.len()..];

                    match sub.find(';') {
                        None => {}
                        Some(pos) => {
                            sub = &sub[..pos];
                        }
                    }

                    match sub.strip_prefix("\"") {
                        None => {}
                        Some(strip) => {
                            sub = strip;
                        }
                    }

                    match sub.strip_suffix("\"") {
                        None => {}
                        Some(strip) => {
                            sub = strip;
                        }
                    }

                    Some(sub.to_owned())
                }
            },
        },
    };

    match oboundary {
        None => {
            return text_response(
                400,
                format!("Boundary not found in {} header.\r\n", header::CONTENT_TYPE),
            );
        }
        Some(boundary) => {
            let mut form = Multipart::with_body(Cursor::new(req.into_body()), boundary);
            loop {
                match form.read_entry() {
                    Err(err) => {
                        return text_response(400, format!("Invalid form data: {}\r\n", err));
                    }
                    Ok(None) => return text_response(400, "CSV missing.\r\n"),
                    Ok(Some(entry)) => match entry.headers.name.as_ref() {
                        "csv" => {
                            http500_unless!(
                                "Failed to init database",
                                db.execute(include_str!("schema.sql"))
                            );

                            http500_unless!(
                                "Failed to init transaction",
                                db.execute("BEGIN IMMEDIATE")
                            );

                            let snapshot_time = http500_unless!(
                                "Failed to travel in time",
                                time::SystemTime::now().duration_since(time::UNIX_EPOCH)
                            )
                            .as_secs();

                            for i in ReaderBuilder::new()
                                .delimiter(b';')
                                .from_reader(entry.data)
                                .records()
                            {
                                match i {
                                    Err(err) => {
                                        return text_response(
                                            400,
                                            format!("Invalid CSV: {}\r\n", err),
                                        );
                                    }
                                    Ok(row) => match row.get(0) {
                                        None => {}
                                        Some(name) => {
                                            let hex_name = HexFmt::new(name.as_bytes());

                                            http500_unless!(
                                                "Failed to INSERT INTO person",
                                                db.execute(format!("INSERT OR IGNORE INTO person(name) VALUES (unhex('{}'))", hex_name))
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
                        _ => {}
                    },
                }
            }
        }
    }
}
