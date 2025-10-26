use cgi::{Request, Response, empty_response, handle};

fn main() {
    handle(handler);
}

fn handler(_: Request) -> Response {
    empty_response(501)
}
