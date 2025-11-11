mod import_memopzk;

use cgi::http::{Method, header};
use cgi::{Request, Response, handle, html_response, text_response};

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

    match req.uri().query() {
        None => match req.method() {
            &Method::GET => html_response(200, include_str!("index.html")),
            _ => text_response(405, "Request method must be GET.\r\n"),
        },
        Some("import-memopzk") => match req.method() {
            &Method::GET => html_response(200, include_str!("import-memopzk.html")),
            &Method::POST => import_memopzk::handler(req),
            _ => text_response(405, "Request method must be GET or POST.\r\n"),
        },
        Some(_) => text_response(404, "No such action.\r\n"),
    }
}
