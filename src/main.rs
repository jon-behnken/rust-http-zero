use crate::response::{Response, http_status_code::HttpStatusCode};
use crate::router::Router;
use crate::server::Server;

mod request;
mod response;
mod router;
mod server;
fn main() {
    // Instantiate router and register route handlers
    let mut router = Router::new();
    router.get(
        "/foo".to_string(),
        Box::new(|_| Response::new(HttpStatusCode::Ok, String::from("hello").into())),
    );
    router.post(
        "/foo".to_string(),
        Box::new(|request| {
            let body = request.body();
            Response::new(HttpStatusCode::Created, body.clone())
        }),
    );

    Server::start(router, 6403, None)
}
