use cgi::{Request, Response, empty_response, handle, http::header, text_response};

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

    empty_response(501)
}
