use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

use hyperflow::HTTP2;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let mut h2 = HTTP2::new(stream);
        h2.recv();
    }
}
