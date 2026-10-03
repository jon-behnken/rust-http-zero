use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, mpsc::Sender};
use std::thread;

use crate::request::Request;
use crate::router::Router;

/**
 * A shared, global handler that closes over a Router and
 * dispatches requests to registered RouteHandlers.
 *
 * A clone of the Dispatcher is moved into every spawned thread,
 * so wrapping it in an Arc prevents redundant allocation of the
 * same type.
 */
type Dispatcher = Arc<dyn Fn(TcpStream) + Send + Sync>;

pub struct ServerOptions {
    dispatcher: Dispatcher,
    channel_message_sender: Option<Sender<()>>,
}
pub struct Server {}

impl Server {
    pub fn start(router: Router, port: u16, options: Option<ServerOptions>) {
        let dispatcher: Dispatcher;
        let mut sender: Option<Sender<()>> = None;
        match options {
            Some(server_options) => {
                dispatcher = server_options.dispatcher;
                sender = server_options.channel_message_sender;
            }
            None => {
                // Move Router into a heap allocation with reference counter
                let threaded_router = Arc::new(router);
                dispatcher = Arc::new(move |mut stream: TcpStream| {
                    let request = Request::from_stream(&mut stream).unwrap(); // FIXME error_handling
                    println!(
                        "[Router] [{:?}] {:?}",
                        request.method(),
                        request.request_target()
                    );
                    threaded_router.dispatch(request);
                });
            }
        }
        Server::start_tcp_listener(dispatcher, sender, port);
    }

    fn start_tcp_listener(dispatcher: Dispatcher, sender: Option<Sender<()>>, port: u16) {
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
                            let threaded_dispatcher: Arc<dyn Fn(TcpStream) + Send + Sync> =
                                dispatcher.clone();
                            // thread::spawn requires ownership but the closure doesn't know that.
                            // We need to explicitly specify `move` capture mode
                            thread::spawn(move || threaded_dispatcher(stream));
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

    fn test_dispatcher(mut stream: TcpStream) {
        thread::sleep(Duration::from_millis(TEST_HANDLER_SLEEP_TIME_MS.into()));
        stream.write(b"done").unwrap();
    }

    #[test]
    fn it_handles_requests_concurrently() {
        let (sender, receiver) = channel::<()>();

        let options = ServerOptions {
            dispatcher: Arc::new(test_dispatcher),
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
