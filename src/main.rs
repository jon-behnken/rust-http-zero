use std::io::Write;
use std::net::TcpListener;
use std::sync::mpsc::{Sender, channel};
use std::thread;

use crate::request::Request;

mod request;

fn start_tcp_listener(sender: Option<Sender<()>>, port: u16) {
    match TcpListener::bind(format!("127.0.0.1:{port}")) {
        Err(e) => println!("Error binding to port {port}: {:?}", e),
        Ok(tcp_listener) => {
            if let Some(sender) = sender {
                sender
                    .send(())
                    .expect("Error sending server started signal");
            };

            for stream in tcp_listener.incoming() {
                match stream {
                    Err(e) => println!("Error with stream: {:?}", e),
                    Ok(stream) => {
                        thread::spawn(|| {
                            /*
                            In Rust, every value has exactly one owner, so when spawning a new thread,
                            `stream` must be moved into it.

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

fn main() {
    start_tcp_listener(None, 6403);
}

#[cfg(test)]
mod test {
    use std::net::TcpStream;

    use super::*;

    #[test]
    fn it_handles_requests_in_threads() {
        let (sender, receiver) = channel::<()>();

        thread::spawn(|| {
            start_tcp_listener(Some(sender), 6113);
        });

        // FIXME clean up all this duplication
        match receiver.recv() {
            Err(e) => println!("Error receiving server started signal: {:?}", e),
            Ok(_) => {
                let (sender, receiver) = channel::<()>();
                let sender_clone = sender.clone();

                thread::spawn(move || {
                    let mut stream = TcpStream::connect("localhost:6113")
                        .expect("Error connecting to localhost on port 6113");
                    stream.write(b"ping").expect("Error sending request");
                    sender.clone().send(())
                });

                thread::spawn(move || {
                    let mut stream = TcpStream::connect("localhost:6113")
                        .expect("Error connecting to localhost on port 6113");
                    stream.write(b"pong").expect("Error sending request");
                    sender_clone.send(())
                });

                match receiver.recv() {
                    Err(e) => println!("Error receiving request sent signal: {:?}", e),
                    Ok(_) => {

                    }

                }
            }
        }
    }
}
