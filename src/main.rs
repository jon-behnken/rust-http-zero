use std::io::Write;
use std::net::TcpListener;
use std::thread;

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
                    Ok(stream) => {
                        thread::spawn(|| {
                            /*
                            In Rust, every value has exactly one owner,
                            so when spawning a new thread, `stream` must be moved into it.

                            spawn() accepts a closure, which is an anonymous function that has
                            access to variables in the enclosing scope. If any captured variables 
                            are borrowed, those variables must be moved into the thread using `move`.

                            In this case, because from_stream() already takes `stream` by value,
                            the `move` keyword is not necessary.
                             */
                            let mut request = Request::from_stream(stream);
                            println!("{}", request.request_target);
                            request
                            .stream_ref()
                            .write_all(
                                b"HTTP/1.1 200 OK\r\nDate: Sat, 05 Sep 2026 19:10:00 GMT\r\n\r\n<!DOCTYPE html><html><body>Hi</body></html>
",
                            )
                            .expect("test")
                        });
                    }
                }
            }
        }
    };
}