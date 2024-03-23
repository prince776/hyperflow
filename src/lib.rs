use core::panic;
use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashMap,
    io::{BufReader, BufWriter, Read, Write},
    net::{self, TcpStream},
    str,
};

use h2::HeaderFlagMask;
use hpack::{Decoder, Encoder};
use http::{Request, Response};
use stream::{Stream, StreamData};

use crate::h2::{Frame, Header};

mod h2;
pub mod http;
mod stream;

pub struct H2 {
    tcp_stream: net::TcpStream,
    streams: HashMap<u32, Stream>,
}

impl H2 {
    const H2PREFACE: &'static str = "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
    const CONN_STREAM_ID: u32 = 0;

    pub fn new(tcp_stream: TcpStream) -> H2 {
        let streams: HashMap<u32, Stream> = HashMap::new();
        H2 {
            tcp_stream,
            streams,
        }
    }

    pub fn start<ReqHandler>(&mut self, req_handler: ReqHandler)
    where
        ReqHandler: Fn(Request) -> Response,
    {
        // TODO: See if we can still use buffered reader on this when needing to write to it.
        // let mut tcp_reader = BufReader::new(self.tcp_stream.borrow());

        if let Err(err) = H2::establish_h2_conn(&mut self.tcp_stream) {
            println!("Error: {}. Abort!", err);
            return;
        }
        println!("HTTP2 Connection Established!");

        let mut encoder = Encoder::new();
        let mut decoder = Decoder::new();
        loop {
            let frame = H2::parse_frame(&mut self.tcp_stream);
            let frame = match frame {
                Ok(v) => v,
                Err(e) => {
                    println!("Error parsing frame: {}", e);
                    return;
                }
            };

            let stream_id = frame.header.stream_identifier();
            if stream_id == H2::CONN_STREAM_ID {
                println!(
                    "Not dealing with stream id 0 frames, got frame: {:#?}",
                    frame
                );
                continue;
            }

            if !self.streams.contains_key(&stream_id) {
                self.streams.insert(stream_id, Stream::new(stream_id));
            }

            let stream = self.streams.remove(&stream_id).unwrap();
            let mut stream = stream.on_frame(&mut decoder, frame);

            if stream.should_handle_request() {
                if let Some(req) = stream.curr_request() {
                    let resp = req_handler(req);
                    stream.send(resp, &mut encoder, &mut self.tcp_stream);
                    stream.set_request_handled();
                }
            }

            self.streams.insert(stream_id, stream);
        }
    }

    fn establish_h2_conn(tcp_stream: &mut TcpStream) -> Result<(), String> {
        let mut preface_buf: [u8; 24] = Default::default();
        let preface_bytes = tcp_stream.read(preface_buf.as_mut_slice()).unwrap();

        if preface_bytes != 24 {
            return Err(format!("Not an http2 connection, aborted!"));
        }

        let preface_str = match std::str::from_utf8(&preface_buf) {
            Ok(v) => v,
            Err(_) => {
                return Err(format!("Invalid preface: {}", preface_bytes));
            }
        };

        if preface_str != H2::H2PREFACE {
            return Err(format!("Not a valid http2 preface. Aborted!"));
        }

        // Send server connection preface, that is an empty settings frame.
        {
            let fheader = h2::Header::from_fields(0, h2::FrameType::Settings, HeaderFlagMask(0), 0);
            let fbody = Vec::new();

            let frame = Frame::new(fheader, fbody);
            let frame_buf = frame.serialize();

            tcp_stream
                .write(&frame_buf)
                .expect("Failed to send server conn preface");
        }

        Ok(())
    }

    fn parse_frame<'a>(reader: &mut impl Read) -> Result<Frame, String> {
        let mut header_buf: [u8; 9] = Default::default();
        let mut bytes_read = reader.read(&mut header_buf).unwrap();
        if bytes_read != 9 {
            return Err(String::from("EOF"));
        }

        let header = Header::new(&header_buf);
        let header = match header {
            Err(e) => {
                return Err(format!("Invalid frame header: {}", e));
            }
            Ok(v) => v,
        };

        let mut body: Vec<u8> = Vec::new();
        body.resize(header.length() as usize, 0u8);
        bytes_read = reader.read(&mut body).unwrap();
        if bytes_read != body.len() {
            return Err(String::from("EOF"));
        }

        Ok(Frame::new(header, body))
    }
}
