use std::io::Write;
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
                        let mut request = Request::from_stream(s);
                        println!("{}", request.request_target);
                        request
                            .stream_ref()
                            .write_all(
                                b"HTTP/1.1 200 OK\r\nDate: Sat, 05 Sep 2026 19:10:00 GMT\r\n\r\n<!DOCTYPE html><html><body>Hi</body></html>
",
                            )
                            .expect("test");
                    }
                }
            }
        }
    };
}
