use std::{collections::HashMap, net::TcpStream};

use crate::{Request, request::HttpMethod};

/* A type alias for readability */
type RequestTarget = String;

type RouterRegistry = HashMap<(HttpMethod, RequestTarget), Box<dyn Fn(Request<TcpStream>) + Send + Sync>>;

pub struct Router {
  pub registry: RouterRegistry
}

impl Router {
  pub fn new() -> Self {
    Self { registry: HashMap::new() }
  }
  pub fn get(&mut self, request_target: RequestTarget, handler: Box<dyn Fn(Request<TcpStream>) + Send + Sync>){
    self.registry.insert((HttpMethod::Get, request_target), handler);
  }
}