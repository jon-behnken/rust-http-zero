use std::io::Write;

use crate::response::Response;
use crate::router::Router;
use crate::server::Server;

mod request;
mod response;
mod router;
mod server;
fn main() {
    // Instantiate router and register route handlers
    let mut router = Router::new();
    router.get("/foo".to_string(), Box::new(|_| Response::new(200, vec![])));
    router.post(
        "/foo".to_string(),
        Box::new(|request| {
            let body = request.body();
            Response::new(200, body.clone())
        }),
    );

    Server::start(router, 6403, None)
}
