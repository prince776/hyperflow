#[derive(Debug)]
pub struct Header {
    pub key: String,
    pub value: String,
}

pub type Headers = Vec<Header>;

#[derive(Debug)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(Debug)]
pub struct Request {
    pub url: String,
    pub method: Method,
    pub headers: Headers,
    pub body: String,
}

impl Request {
    pub fn new() -> Request {
        Request {
            url: String::new(),
            method: Method::GET,
            headers: Vec::new(),
            body: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub headers: Headers,
    pub body: Vec<u8>,
}
