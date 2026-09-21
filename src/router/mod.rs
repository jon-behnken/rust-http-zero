use std::{collections::HashMap, net::TcpStream};

use crate::{Request, request::HttpMethod};

/* A type alias for readability */
type RequestTarget = String;

type Router = HashMap<(HttpMethod, RequestTarget), Box<dyn Fn(Request<TcpStream>)>>;