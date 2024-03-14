use hpack::{Decoder, Encoder};

use crate::{
    h2::{Frame, FrameType, Header, HeaderFlag},
    http::{self, Request},
};

pub struct StreamData {
    id: u32,
    curr_request: Option<Request>,
}

pub enum Stream {
    Idle(StreamData),
    Open(StreamData),
    Closed(StreamData),
}

impl Stream {
    pub fn id(&self) -> u32 {
        match self {
            Stream::Idle(d) => d.id,
            Stream::Open(d) => d.id,
            Stream::Closed(d) => d.id,
        }
    }

    pub fn curr_request(&mut self) -> Option<Request> {
        match self {
            Stream::Idle(d) => {
                if let Some(req) = d.curr_request.take() {
                    return Some(req);
                }
            }
            Stream::Open(d) => {
                if let Some(req) = d.curr_request.take() {
                    return Some(req);
                }
            }
            Stream::Closed(d) => {
                if let Some(req) = d.curr_request.take() {
                    return Some(req);
                }
            }
        }
        None
    }
}

impl Stream {
    pub fn new(id: u32) -> Stream {
        println!("Started new idle stream");
        Stream::Idle(StreamData {
            id,
            curr_request: None,
        })
    }
    pub fn on_frame(self, decoder: &mut Decoder, frame: Frame) -> Stream {
        println!("Stream {} processing frame", self.id());

        match self {
            Stream::Idle(data) => Stream::on_frame_idle(frame, decoder, data),
            _ => {
                unimplemented!();
            }
        }
    }

    fn on_frame_idle(frame: Frame, decoder: &mut Decoder, mut data: StreamData) -> Stream {
        match frame.header.frame_type() {
            FrameType::Headers => {}
            _ => {
                println!("Idle stream state does not accept non header frames");
                return Stream::Idle(data);
            }
        }
        if !frame.header.flag_mask().check(HeaderFlag::EndHeaders) {
            unimplemented!("Headers frame without end header is not supported");
        }

        let decoded_headers = decoder
            .decode(&frame.body)
            .expect("Failed to decode headers");

        let mut headers = Vec::new();
        for (key, value) in decoded_headers {
            headers.push(http::Header {
                key: String::from_utf8_lossy(&key).into_owned(),
                value: String::from_utf8_lossy(&value).into_owned(),
            });
        }

        let mut req: Request;
        // Consume unprocessed request, do not buffer.
        if let Some(curr_req) = data.curr_request {
            req = curr_req;
        } else {
            req = Request::new();
        }
        req.headers = headers;

        data.curr_request = Some(req);

        Stream::Open(data)
    }
}
