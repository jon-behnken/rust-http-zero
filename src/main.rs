use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, mpsc::Sender};
use std::thread;

use crate::request::Request;

mod request;

/*
RequestHandler is a handler function wrapped in an atomic reference counter.

The atomic reference counter allows a reference to the handler to be safely
shared across threads; the difference between Rc and Arc is that an Arc
guarantees that the read and write operations to the reference count are
done atomically.
*/
type RequestHandler = Arc<dyn Fn(TcpStream) + Send + Sync>;

fn request_handler(stream: TcpStream) {
    /*
    spawn() accepts a closure, which is an anonymous function that has
    access to variables in the enclosing scope. To avoid dangling references,
    Rust normally requires you to `move` values into the closure.

    However, in this case, the `move` keyword is not necessary because
    the closure fully consumes `stream`.
     */
    let mut request = Request::from_stream(stream);
    println!("{}", request.request_target);
    request
        .stream_ref()
        .write_all(
            b"HTTP/1.1 200 OK\r\nDate: Sat, 05 Sep 2026 19:10:00 GMT\r\n\r\n<!DOCTYPE html><html><body>Hi</body></html>",
        )
        .expect("{WIP}") // FIXME
}

fn start_tcp_listener(handler: RequestHandler, sender: Option<Sender<()>>, port: u16) {
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
                        let clone = handler.clone();
                        thread::spawn(move || clone(stream));
                    }
                }
            }
        }
    };
}

fn main() {
    start_tcp_listener(Arc::new(request_handler), None, 6403);
}

#[cfg(test)]
mod test {
    use std::io::Read;
    use std::net::TcpStream;
    use std::panic;
    use std::sync::mpsc::channel;
    use std::time::{Duration, Instant};

    use super::*;

    const TEST_HANDLER_SLEEP_TIME_MS: u16 = 500;
    const THREAD_OVERHEAD_BUFFER_MS: u16 = 100;
    const EXPECTED_DURATION_THRESHOLD_MS: u16 =
        TEST_HANDLER_SLEEP_TIME_MS + THREAD_OVERHEAD_BUFFER_MS;

    fn test_handler(mut stream: TcpStream) {
        thread::sleep(Duration::from_millis(TEST_HANDLER_SLEEP_TIME_MS.into()));
        stream.write(b"done").unwrap();
    }

    #[test]
    fn it_handles_requests_in_threads() {
        let (sender, receiver) = channel::<()>();

        thread::spawn(|| {
            start_tcp_listener(Arc::new(test_handler), Some(sender), 6113);
        });

        // Start test once server starts up successfully
        match receiver.recv() {
            Err(e) => println!("Error receiving server started signal: {:?}", e),
            Ok(_) => {
                let (sender, receiver) = channel::<()>();
                let sender_clone = sender.clone();

                // send_request takes ownership of sender and returns a closure which
                // captures it. The closure outlives the scope of send_request,
                // which would result in a dangling reference (sender would be dropped).
                //
                // This is where Rust requires the `move` keyword -- to force ownership
                // into the closure.
                fn send_request(sender: Sender<()>) -> impl FnOnce() {
                    move || {
                        let mut stream = TcpStream::connect("localhost:6113")
                            .expect("Error connecting to localhost on port 6113");
                        stream.write(b"ping").expect("Error sending request");

                        let mut response = [0; 4];
                        stream.read(&mut response).unwrap();
                        if let Err(e) = sender.send(()) {
                            println!("{:?}", e);
                            panic!()
                        }
                    }
                }

                let start = Instant::now();

                thread::spawn(send_request(sender));
                thread::spawn(send_request(sender_clone));

                match receiver.recv() {
                    Err(e) => println!("Error receiving request sent signal: {:?}", e),
                    Ok(_) => {
                        let duration = start.elapsed();
                        assert!(duration.as_millis() < EXPECTED_DURATION_THRESHOLD_MS.into());
                    }
                }
            }
        }
    }
}
