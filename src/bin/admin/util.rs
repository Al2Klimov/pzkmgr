use cgi::{Request, Response, http::header, text_response};
use multipart::server::{Multipart, MultipartData};
use sqlite::Connection;
use std::io::Cursor;

#[macro_export]
macro_rules! http500_unless {
    ($errmsg:expr, $result:expr) => {
        match $result {
            Ok(v) => v,
            Err(err) => {
                return text_response(500, format!("{}: {}\r\n", $errmsg, err));
            }
        }
    };
}

pub(crate) type UploadedFile<'a> = MultipartData<&'a mut Multipart<Cursor<Vec<u8>>>>;

pub(crate) fn handle_upload(
    db: Connection,
    req: Request,
    file_handler: impl FnOnce(Connection, UploadedFile) -> Response,
) -> Response {
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
                    Ok(None) => return text_response(400, "File missing.\r\n"),
                    Ok(Some(entry)) => match entry.headers.name.as_ref() {
                        "file" => return file_handler(db, entry.data),
                        _ => {}
                    },
                }
            }
        }
    }
}
