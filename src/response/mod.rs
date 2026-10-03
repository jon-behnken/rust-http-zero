pub struct Response {
    status_code: u16,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status_code: u16, body: Vec<u8>) -> Self {
        Self { status_code, body }
    }
}
