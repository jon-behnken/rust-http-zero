use std::{collections::HashMap, io::Write, net::TcpStream};

use crate::request::{Request, http_method::HttpMethod};

/* A type alias for readability */
type RequestTarget = String;

/** Using a tuple as they key for a HashMap has interesting consequences.
*  HashMap::get accepts anything that implements Borrow<K> where K is the key type.
*  For example,
*  ```
*    let h: HashMap<String, String> = HashMap::new();
     h.get("hello");
*  ```
*  This works because String implements Borrow<str> and returns &str.
*
*  A tuple doesn't implement Borrow, so when clients use HashMap::get
*  they are forced to transiently clone the HttpMethod variant and
*  RequestTarget string; the performance cost is trivial and acceptable
*  and avoiding unnecessary clones introduces more complexity here than
*  the advantage is worth.
*/
type RouterRegistry =
    HashMap<(HttpMethod, RequestTarget), Box<dyn Fn(Request<TcpStream>) + Send + Sync>>;
pub struct Router {
    registry: RouterRegistry,
}

impl Router {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }
    pub fn dispatch(&self, mut request: Request<TcpStream>) {
        let method = request.method();
        let request_target = request.request_target();
        match self
            .registry
            .get(&(method.clone(), request_target.to_string()))
        {
            Some(handler) => handler(request),
            None => {
                request.stream_ref().write_all(b"HTTP/1.1 404 Not Found\r\nDate: Sat, 05 Sep 2026 19:10:00 GMT\r\n\r\n<!DOCTYPE html><html><body>Foo</body></html>").unwrap(); // FIXME error_handling
            }
        }
    }

    pub fn get(
        &mut self,
        request_target: RequestTarget,
        handler: Box<dyn Fn(Request<TcpStream>) + Send + Sync>,
    ) {
        self.registry
            .insert((HttpMethod::Get, request_target), handler);
    }
}
