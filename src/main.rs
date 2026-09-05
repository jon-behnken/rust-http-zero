use std::net::TcpListener;

use crate::request::Request;

mod request;

const PORT: u16 = 6403;

fn main() {
    match TcpListener::bind(format!("127.0.0.1:{PORT}")) {
        Err(e) => println!("Error binding to port {PORT}: {:?}", e),
        Ok(tcp_listener) => {
            for stream in tcp_listener.incoming() {
                match stream {
                    Err(e) => println!("Error with stream: {:?}", e),
                    Ok(s) => {
                        let request = Request::from_stream(s);
                        println!("{:#?}", request.headers);
                        println!("{:#?}", String::from_utf8(request.body));
                    }
                }
            }
        }
    };
}
