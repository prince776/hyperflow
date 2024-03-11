use std::{
    io::Read,
    net::{self, TcpStream},
};

pub struct HTTP2 {
    tcp_stream: net::TcpStream,
}

impl HTTP2 {
    const H2PREFACE: &'static str = "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

    pub fn new(tcp_stream: TcpStream) -> HTTP2 {
        return HTTP2 { tcp_stream };
    }

    pub fn recv(&mut self) {
        // let buf_reader = BufReader::new(self.tcp_stream);
        // let data: Vec<_> = buf_reader.lines().map(|result| result.unwrap()).collect();
        // for line in data {
        //     println!("{}", line);
        // }

        let mut preface_buf: [u8; 24] = Default::default();
        let preface_bytes = self.tcp_stream.read(preface_buf.as_mut_slice()).unwrap();

        if preface_bytes != 24 {
            println!("Not a http2 connection, aborted!");
            return;
        }

        let preface_str = match std::str::from_utf8(&preface_buf) {
            Ok(v) => v,
            Err(_) => {
                println!("Invalid preface: {}", preface_bytes);
                return;
            }
        };

        if preface_str != HTTP2::H2PREFACE {
            println!("Not a valid http2 preface. Aborted!");
            return;
        }
        println!("HTTP2 connection started");
    }
}
