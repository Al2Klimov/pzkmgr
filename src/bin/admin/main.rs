#![recursion_limit = "512"]

mod hex_fmt;
mod import_memopzk;
mod import_vcard;
mod list_current;
mod nullint_fmt;
mod util;

use cgi::http::{Method, header};
use cgi::{Request, Response, handle, html_response, text_response};
use sqlite;
use std::env::var_os;

fn main() {
    handle(handler);
}

fn handler(req: Request) -> Response {
    if !req.headers().contains_key(header::AUTHORIZATION) {
        return text_response(
            511,
            format!(
                "\"{}\" header missing.\r\nThe webserver seems not to restrict access to this admin interface.\r\nRefusing operation!\r\n",
                header::AUTHORIZATION
            ),
        );
    }

    const SEC_FETCH_SITE: &str = "sec-fetch-site";
    const SAME_ORIGIN: &str = "same-origin";
    const NONE: &str = "none";

    let csrf = req.headers().get_all(SEC_FETCH_SITE);

    if csrf.iter().ne([SAME_ORIGIN]) && csrf.iter().ne([NONE]) {
        return text_response(
            403,
            format!(
                "\"{}\" header value is none of: {}, {}\r\nEither the browser doesn't support it or this is a CSRF attack.\r\nRefusing operation!\r\n",
                SEC_FETCH_SITE, SAME_ORIGIN, NONE
            ),
        );
    }

    const PZKMGR_DB: &str = "PZKMGR_DB";

    let mut db = match var_os(PZKMGR_DB) {
        None => {
            return text_response(500, format!("Env var \"{}\" missing.\r\n", PZKMGR_DB));
        }
        Some(path) => http500_unless!("Failed to open database", sqlite::open(path)),
    };

    http500_unless!(
        "Failed to set database lock timeout",
        db.set_busy_timeout(4096)
    );

    http500_unless!(
        "Failed to enforce foreign keys",
        db.execute("PRAGMA foreign_keys = ON")
    );

    match req.uri().query() {
        None => match req.method() {
            &Method::GET => html_response(200, include_str!("index.html")),
            _ => text_response(405, "Request method must be GET.\r\n"),
        },
        Some("import-memopzk") => match req.method() {
            &Method::GET => html_response(200, include_str!("import-memopzk.html")),
            &Method::POST => util::handle_upload(db, req, import_memopzk::handler),
            _ => text_response(405, "Request method must be GET or POST.\r\n"),
        },
        Some("import-vcard") => match req.method() {
            &Method::GET => html_response(200, include_str!("import-vcard.html")),
            &Method::POST => util::handle_upload(db, req, import_vcard::handler),
            _ => text_response(405, "Request method must be GET or POST.\r\n"),
        },
        Some("list-current") => match req.method() {
            &Method::GET => list_current::handler(db, req),
            _ => text_response(405, "Request method must be GET.\r\n"),
        },
        Some(_) => text_response(404, "No such action.\r\n"),
    }
}
