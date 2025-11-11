use cgi::{Request, Response, http::header, text_response};
use csv::ReaderBuilder;
use multipart::server::Multipart;
use std::io::Cursor;

pub(crate) fn handler(req: Request) -> Response {
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
                                        Some(name) => eprintln!("Name: '{}'", name),
                                    },
                                }
                            }

                            todo!();
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}
