use core::fmt;
use std::fmt::{format, Display};

#[derive(Debug, Clone)]
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

impl<'a> Method {
    pub fn from(s: &str) -> Result<Method, &'a str> {
        match s {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "PATCH" => Ok(Method::PATCH),
            _ => Err("Invalid HTTP Method"),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub url: URL,
    pub method: Method,
    pub headers: Headers,
    pub len: u32,
    body: Vec<u8>,
}

#[derive(Debug)]
pub struct URL {
    scheme: String,
    host: String,
    path: String,
}

impl URL {
    pub fn empty() -> URL {
        URL {
            scheme: String::new(),
            host: String::new(),
            path: String::new(),
        }
    }
    pub fn to_string(&self) -> String {
        format!("{}://{}{}", self.scheme, self.host, self.path)
    }
}

impl fmt::Display for URL {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl<'a> Request {
    pub fn empty() -> Request {
        Request {
            url: URL::empty(),
            method: Method::GET,
            headers: Vec::new(),
            len: 0,
            body: Vec::new(),
        }
    }

    pub fn from_h2_headers(h2headers: Headers) -> Result<Request, &'a str> {
        let mut h1_1headers = Vec::new();
        let mut req = Request::empty();

        for header in h2headers {
            match header.key.as_str() {
                ":path" => {
                    req.url.path = header.value;
                }
                ":authority" => {
                    req.url.host = header.value;
                }
                ":scheme" => {
                    req.url.scheme = header.value;
                }
                ":method" => {
                    req.method = Method::from(&header.value)?;
                }
                "content-length" => {
                    req.len = header.value.parse::<u32>().unwrap();
                }
                _ => {
                    h1_1headers.push(header);
                }
            }
        }

        Ok(req)
    }

    pub fn set_body(&mut self, buf: Vec<u8>) {
        assert!(self.len == buf.len() as u32);
        self.body = buf
    }

    pub fn body(&self) -> &Vec<u8> {
        &self.body
    }
}

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub headers: Headers,
    pub body: Vec<u8>,
}

impl Response {
    pub fn h2headers(&self) -> Headers {
        let mut h2headers = self.headers.to_vec();
        h2headers.push(Header {
            key: String::from(":status"),
            value: self.status.to_string(),
        });
        h2headers.push(Header {
            key: String::from("content-length"),
            value: self.body.len().to_string(),
        });

        h2headers
    }
}
