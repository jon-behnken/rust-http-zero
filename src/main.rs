use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, mpsc::Sender};
use std::thread;

use crate::request::Request;
use crate::router::Router;

mod request;
mod router;

/*
RequestHandler is a handler function wrapped in an atomic reference counter.

An Arc guarantees that the read and write operations to the reference count
are done atomically, thus its safe to share across threads.
*/
type RequestHandler = Arc<dyn Fn(TcpStream) + Send + Sync>;

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
                        let clone: Arc<dyn Fn(TcpStream) + Send + Sync> = handler.clone();
                        // thread::spawn requires ownership but the closure doesn't know that.
                        // We need to explicitly specify `move` capture mode
                        thread::spawn(move || clone(stream));
                    }
                }
            }
        }
    };
}

fn main() {
    // Instantiate router and register route handlers
    let mut router = Router::new();
    router.get(
        "/foo".to_string(),
        Box::new(|mut request| {
            request.stream_ref().write_all(b"HTTP/1.1 200 OK\r\nDate: Sat, 05 Sep 2026 19:10:00 GMT\r\n\r\n<!DOCTYPE html><html><body>Foo</body></html>").unwrap();
        }),
    );
    router.get(
        "/bar".to_string(),
        Box::new(|mut request| {
            request.stream_ref().write_all(b"HTTP/1.1 200 OK\r\nDate: Sat, 05 Sep 2026 19:10:00 GMT\r\n\r\n<!DOCTYPE html><html><body>Bar</body></html>").unwrap();
        }),
    );

    // Move Router into a heap allocation with reference counter
    let threaded_router = Arc::new(router);
    start_tcp_listener(
        Arc::new(move |stream: TcpStream| {
            let request = Request::from_stream(stream).unwrap(); // FIXME error_handling
            println!(
                "[Router] [{:?}] {:?}",
                request.method(),
                request.request_target()
            );
            threaded_router.dispatch(request);
        }),
        None,
        6403,
    );
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
    fn it_handles_requests_concurrently() {
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

                receiver.recv().and_then(|_| receiver.recv()).unwrap();
                let duration = start.elapsed();
                assert!(duration.as_millis() < EXPECTED_DURATION_THRESHOLD_MS.into());
            }
        }
    }
}
