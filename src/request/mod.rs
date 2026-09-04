use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;

#[derive(Debug, Clone)]
pub struct Headers {
    pub content_length: Option<u64>,
    pub content_type: Option<String>,
    pub accept: Option<String>,
    pub host: Option<String>,
    pub user_agent: Option<String>,
}
pub struct Request {
    stream: TcpStream,
    pub headers: Headers,
    pub body: Option<Vec<u8>>,
}

const HTTP_HEADER_TERMINATOR: &[u8; 4] = b"\r\n\r\n";

impl Request {
    pub fn from_stream(stream: TcpStream) -> Self {
        Request::consume(stream)
    }

    fn set_headers(&mut self) {
        // A top-level buffer to store the full headers read from the stream
        let mut headers_buffer: Vec<u8> = vec![];
        let mut headers_read = false;

        while !headers_read {
            // A scoped buffer that continuously reads bytes off the stream and pushes them into the top-level buffer
            let mut buf: [u8; 512] = [0; 512];
            match self.stream.read(&mut buf) {
                Err(e) => println!("Error reading into buffer: {:?}", e),
                Ok(bytes_read) => {
                    let bytes = &buf[0..bytes_read];
                    headers_buffer.extend(bytes);
                    for (i, window) in headers_buffer.windows(4).enumerate() {
                        if window == HTTP_HEADER_TERMINATOR {
                            headers_read = true;
                            // If we've read past the header, store any remaining bytes in the body
                            self.body = Some(headers_buffer[i + 4..].to_vec());
                        }
                    }
                }
            }
        }

        let mut headers: HashMap<String, String> = HashMap::new();
        let mut header_key: Vec<u8> = vec![];
        let mut header_value: Vec<u8> = vec![];

        // Advance to the next key-value pair in headers
        let mut write_to_header_value = false;

        let mut i: usize = 0;
        while i < headers_buffer.len() - 2 {
            let advance_header = &headers_buffer[i..i + 2] == b"\r\n";

            // Transition to writing the header's value
            // Skip 2 bytes to exclude ': '
            // FIXME: there's a bug here (e.g. localhost:6403 becomes localhost403 in 'Host' header)
            if headers_buffer[i] == b':' {
                write_to_header_value = true;
                i += 2;
                continue;
            }

            // Insert current header key and value
            // Clear the buffer variables
            // Skip 2 bytes to exclude '\r\n'
            if advance_header {
                headers.insert(
                    String::from_utf8(header_key.clone()).unwrap(),
                    String::from_utf8(header_value.clone()).unwrap(),
                );

                header_key.clear();
                header_value.clear();
                write_to_header_value = false;
                i += 2;
                continue;
            }

            if write_to_header_value {
                header_value.push(headers_buffer[i]);
            } else {
                header_key.push(headers_buffer[i]);
            }

            i += 1;
        }
        self.headers = Headers {
            content_length: headers.remove("Content-Length").map(|h| {
                h.parse::<u64>()
                    .expect("Error parsing Content-Length header")
            }),
            content_type: headers.remove("Content-Type"),
            accept: headers.remove("Accept"),
            host: headers.remove("Host"),
            user_agent: headers.remove("User-Agent"),
        };
    }

    fn consume(stream: TcpStream) -> Self {
        let mut request = Request {
            stream,
            headers: Headers {
                content_length: None,
                content_type: None,
                accept: None,
                host: None,
                user_agent: None,
            },
            body: None,
        };

        request.set_headers();

        let mut total_bytes_read: usize = request.body.as_mut().unwrap().len();
        while total_bytes_read
            < request
                .headers
                .content_length
                .expect("Empty content length")
                .try_into()
                .expect("Error converting content-length to u64")
        {
            let mut buf: [u8; 512] = [0; 512];
            match request.stream.read(&mut buf) {
                Err(e) => println!("Error reading stream body: {:?}", e),
                Ok(bytes_read) => {
                    let bytes = &buf[0..bytes_read];
                    request.body.as_mut().unwrap().extend(bytes);
                    total_bytes_read += bytes_read;
                }
            }
        }
        request
    }
}
