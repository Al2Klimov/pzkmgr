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

    let mut reader = ReaderBuilder::new().from_reader(file);

    let headers = match reader.headers() {
        Err(err) => return text_response(400, format!("Invalid CSV: {}\r\n", err)),
        Ok(headers) => headers.clone(),
    };

    let col = |name: &str| headers.iter().position(|h| h.trim() == name);

    let (name_col, url_col, list_col, birthday_col) = match (
        col("ФИО"),
        col("Ссылка"),
        col("Список Мемориала"),
        col("ДР"),
    ) {
        (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
        _ => return text_response(400, "Invalid CSV: missing required columns\r\n"),
    };

    for i in reader.records() {
        match i {
            Err(err) => return text_response(400, format!("Invalid CSV: {}\r\n", err)),
            Ok(row) => match row.get(name_col) {
                None => {}
                Some(_) if row.get(list_col).map(str::trim) != Some("Общий список") => {}
                Some(name) => {
                    let hex_name = HexFmt::new(name.as_bytes());

                    http500_unless!(
                        "Failed to INSERT INTO person",
                        db.execute(format!(
                            "INSERT OR IGNORE INTO person(name) VALUES (CAST(unhex('{}') AS TEXT))",
                            hex_name
                        ))
                    );

                    let url = row.get(url_col).map(str::trim).unwrap_or("");
                    if !url.is_empty() {
                        http500_unless!(
                            "Failed to UPDATE person",
                            db.execute(format!(
                                "UPDATE person SET url = CAST(unhex('{}') AS TEXT) WHERE name = CAST(unhex('{}') AS TEXT) AND url IS NULL",
                                HexFmt::new(url.as_bytes()),
                                hex_name
                            ))
                        );
                    }

                    let birthday = row
                        .get(birthday_col)
                        .map(|b| {
                            b.trim()
                                .split('/')
                                .map(|p| p.parse::<u32>().ok())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    if let [Some(d), Some(m), Some(y)] = birthday[..] {
                        if (1..=31).contains(&d)
                            && (1..=12).contains(&m)
                            && (1000..=9999).contains(&y)
                        {
                            http500_unless!(
                                "Failed to UPDATE person",
                                db.execute(format!(
                                    "UPDATE person SET birth_day = {}, birth_month = {}, birth_year = {} WHERE name = CAST(unhex('{}') AS TEXT)",
                                    d, m, y, hex_name
                                ))
                            );
                        }
                    }

                    http500_unless!(
                        "Failed to INSERT INTO pzk",
                        // Duplicate name in input => same person_id twice => INSERT OR IGNORE
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
