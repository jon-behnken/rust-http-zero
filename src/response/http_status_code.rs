#[derive(Clone)]
#[repr(u16)]
pub enum HttpStatusCode{
    Ok = 200,
    Created = 201,
    NotFound = 404
}

impl HttpStatusCode{
    pub fn reason(status_code: &HttpStatusCode) -> String {
        match status_code {
            HttpStatusCode::Ok => String::from("OK"),
            HttpStatusCode::Created => String::from("Created"),
            HttpStatusCode::NotFound => String::from("Not Found")
        }
    }
}