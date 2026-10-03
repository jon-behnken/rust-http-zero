use std::collections::HashMap;

use crate::{
    request::{Request, http_method::HttpMethod},
    response::Response,
};

/* A type aliases for readability */
type RequestTarget = String;
type RouteHandler = Box<dyn Fn(&Request) -> Response + Send + Sync>;

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
*  they are forced to transiently clone the relevant HttpMethod variant and
*  RequestTarget string; the performance cost is trivial and acceptable
*  and avoiding unnecessary clones introduces more complexity here than
*  the advantage is worth.
*/
type RouterRegistry = HashMap<(HttpMethod, RequestTarget), RouteHandler>;
pub struct Router {
    registry: RouterRegistry,
}

impl Router {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }
    pub fn dispatch(&self, request: Request) -> Response {
        let method = request.method();
        let request_target = request.request_target();
        match self
            .registry
            .get(&(method.clone(), request_target.to_string()))
        {
            Some(handler) => handler(&request),
            None => Response::new(404, vec![]),
        }
    }

    fn register(
        &mut self,
        http_method: HttpMethod,
        request_target: RequestTarget,
        handler: RouteHandler,
    ) {
        self.registry.insert((http_method, request_target), handler);
    }

    pub fn get(&mut self, request_target: RequestTarget, handler: RouteHandler) {
        self.register(HttpMethod::Get, request_target, handler);
    }

    pub fn post(&mut self, request_target: RequestTarget, handler: RouteHandler) {
        self.register(HttpMethod::Post, request_target, handler);
    }
}
