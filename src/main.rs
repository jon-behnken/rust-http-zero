use std::net::TcpListener;

use crate::request::Request;

mod request;

fn main() {
    let port: u16 = 6403;
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).expect("Error binding to port");

    for stream in listener.incoming() {
        // let mut request: Vec<u8> = vec![];
        match stream {
            Err(e) => println!("Error with stream: {:?}", e),
            Ok(s) => {
                let mut request = Request::new(s);
                request.set_headers();
                println!("{:?}", request.headers);
            }
        }
    }
}
