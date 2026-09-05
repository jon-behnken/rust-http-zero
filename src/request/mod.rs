use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;

#[derive(Debug, Clone)]
pub struct Headers {
    pub authorization: Option<String>,
    pub content_length: Option<u64>,
    pub content_type: Option<String>,
    pub accept: Option<String>,
    pub host: Option<String>,
    pub user_agent: Option<String>,
}

pub struct Request {
    stream: TcpStream,
    pub headers: Headers,
    pub method: String,
    pub body: Vec<u8>,
}

const HTTP_HEADER_TERMINATOR: &[u8; 4] = b"\r\n\r\n";

impl Request {
    /*
       from_stream is the public constructor; it takes a TcpStream
       and calls consume() which reads bytes off the stream and
       parses them into headers, method and body
    */
    pub fn from_stream(stream: TcpStream) -> Self {
        Request::consume(stream)
    }

    /*
       new() is a private constructor that reads the full HTTP headers
       into a buffer, as well as any residual bytes that belong to the body.
    */
    fn new(mut stream: TcpStream) -> Request {
        // Stores all bytes that belong to the header
        let mut headers_buffer: Vec<u8> = vec![];

        // Stores any bytes that are read beyond the HTTP_HEADER_TERMINATOR
        let mut body_buffer: Vec<u8> = vec![];

        let mut headers_read = false;

        while !headers_read {
            // This buffer is scoped to each iteration of the loop;
            // it reads a fixed-size number of bytes off the stream.
            //
            // We don't know ahead of time how large the header will be,
            // so we loop over the stream bytes and fill this buffer each time.
            // Once the scoped buffer is filled, we push those bytes onto the
            // top-level headers_buffer.
            let mut buf: [u8; 512] = [0; 512];
            match stream.read(&mut buf) {
                Err(e) => println!("Error reading into buffer: {:?}", e),
                Ok(bytes_read) => {
                    // The stream may end up containing less bytes than we've allocated for the buffer.
                    // To ensure we only write valid data, slice the buffer from beginning to the last valid byte.
                    let bytes = &buf[0..bytes_read];
                    headers_buffer.extend(bytes);

                    // The HTTP_HEADER_TERMINATOR may arrive in sequential reads (e.g. one call may have '\r' and the next the remaining '\n\r'\n )
                    // So we need to scan the full headers_buffer, not just the scoped one.
                    let mut header_end_index: usize = 0;
                    for (i, window) in headers_buffer.windows(4).enumerate() {
                        if window == HTTP_HEADER_TERMINATOR {
                            headers_read = true;
                            header_end_index = i + 4;

                            // Again, we're reading a fixed number of bytes off the stream each time.
                            // It's possible to read past the header, and into the body. Any bytes that
                            // belong to the body must be stored in a buffer as well.
                            body_buffer = headers_buffer[header_end_index..].to_vec();
                        }
                    }
                    if headers_read {
                        headers_buffer = headers_buffer[0..header_end_index].to_vec()
                    }
                }
            }
        }

        let mut headers: HashMap<String, String> = HashMap::new();
        let mut header_key: Vec<u8> = vec![];
        let mut header_value: Vec<u8> = vec![];

        let mut write_to_http_method = true;

        // Advance to the next key-value pair in headers
        let mut write_to_header_value = false;

        let mut method: Vec<u8> = vec![];
        let mut i: usize = 0;
        while i < headers_buffer.len() - 2 {
            if &headers_buffer[i..i + 2] == b" /" {
                write_to_http_method = false;
                i += 2;
                continue;
            }

            if write_to_http_method {
                method.push(headers_buffer[i]);
                i += 1;
                continue;
            };

            let advance_header = &headers_buffer[i..i + 2] == b"\r\n";

            // Transition to writing the header's value
            // Skip 2 bytes to exclude ': '
            if &headers_buffer[i..i + 2] == b": " {
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

        let headers = Headers {
            authorization: headers.remove("Authorization"),
            content_length: headers.remove("Content-Length").map(|h| {
                h.parse::<u64>()
                    .expect("Error parsing Content-Length header")
            }),
            content_type: headers.remove("Content-Type"),
            accept: headers.remove("Accept"),
            host: headers.remove("Host"),
            user_agent: headers.remove("User-Agent"),
        };

        Request {
            stream,
            headers,
            method: String::from_utf8(method).expect("Error parsing HTTP method"),
            body: body_buffer,
        }
    }

    fn consume(stream: TcpStream) -> Self {
        let mut request = Request::new(stream);

        if request.method == "GET" {
            return request;
        };

        // read() doesn't return until an EOF signal is read; HTTP requests don't send EOF
        // terminators, so in order to read the body, we need to know the Content-Length
        let mut total_bytes_read: usize = request.body.len();
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
                    request.body.extend(bytes);
                    total_bytes_read += bytes_read;
                }
            }
        }
        request
    }
}
