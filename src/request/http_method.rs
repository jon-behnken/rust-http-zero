use std::str::FromStr;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HttpMethod {
    Get,
    Options,
    Post,
    Put,
    Patch,
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "GET" => Ok(HttpMethod::Get),
            "OPTIONS" => Ok(HttpMethod::Options),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "PATCH" => Ok(HttpMethod::Patch),
            _ => Err(format!("Unknown HTTP method encountered: {}", input)),
        }
    }
}
