use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

use hyperflow::{http::Response, H2};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let mut h2 = H2::new(stream);
        h2.start(|req| {
            println!(
                "Handling request: headers: {:#?}, url: {:#?} ({}), method: {:#?}, body len: {}",
                req.headers,
                req.url,
                req.url,
                req.method,
                req.body().len(),
            );

            let body_str = String::from("<Html>Hello</Html>");
            return Response {
                status: 200,
                headers: Vec::new(),
                body: Vec::from(body_str),
            };
        });
        println!("HTTP connection end");
    }
}
