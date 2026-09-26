use std::io::Write;

use crate::router::Router;
use crate::server::Server;

mod request;
mod router;
mod server;
fn main() {
    // Instantiate router and register route handlers
    let mut router = Router::new();
    router.get(
        "/foo".to_string(),
        Box::new(|mut request| {
            request.stream_ref().write_all(b"HTTP/1.1 200 OK\r\nDate: Sat, 05 Sep 2026 19:10:00 GMT\r\n\r\n<!DOCTYPE html><html><body>Foo</body></html>").unwrap();
        }),
    );
    router.get(
        "/bar".to_string(),
        Box::new(|mut request| {
            request.stream_ref().write_all(b"HTTP/1.1 200 OK\r\nDate: Sat, 05 Sep 2026 19:10:00 GMT\r\n\r\n<!DOCTYPE html><html><body>Bar</body></html>").unwrap();
        }),
    );

    Server::start(router, 6403, None)
}
