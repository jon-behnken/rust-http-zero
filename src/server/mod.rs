use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, mpsc::Sender};
use std::thread;

use crate::request::Request;
use crate::router::Router;

/**
 * RequestHandler is a handler function wrapped in an atomic reference counter.
 *
 * An Arc guarantees that the read and write operations to the reference count
 * are done atomically, thus its safe to share across threads.
 */
type RequestHandler = Arc<dyn Fn(TcpStream) + Send + Sync>;

pub struct ServerOptions {
    request_handler: RequestHandler,
    channel_message_sender: Option<Sender<()>>,
}
pub struct Server {}

impl Server {
    pub fn start(router: Router, port: u16, options: Option<ServerOptions>) {
        let request_handler: Arc<dyn Fn(TcpStream) + Send + Sync>;
        let mut sender: Option<Sender<()>> = None;
        match options {
            Some(server_options) => {
                request_handler = server_options.request_handler;
                sender = server_options.channel_message_sender;
            }
            None => {
                // Move Router into a heap allocation with reference counter
                let threaded_router = Arc::new(router);
                request_handler = Arc::new(move |stream: TcpStream| {
                    let request = Request::from_stream(stream).unwrap(); // FIXME error_handling
                    println!(
                        "[Router] [{:?}] {:?}",
                        request.method(),
                        request.request_target()
                    );
                    threaded_router.dispatch(request); // FIXME do I need the threaded one here?
                });
            }
        }
        Server::start_tcp_listener(request_handler, sender, port);
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
}

#[cfg(test)]
mod test {
    use std::io::{Read, Write};
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

        let options = ServerOptions {
            request_handler: Arc::new(test_handler),
            channel_message_sender: Some(sender),
        };

        let router = Router::new();

        thread::spawn(|| {
            Server::start(router, 6113, Some(options));
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
