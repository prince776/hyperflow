use std::io::Write;

use hpack::{Decoder, Encoder};

use crate::{
    h2::{self, Frame, FrameType, Header, HeaderFlag, HeaderFlagMask},
    http::{self, Request, Response},
};

pub struct StreamData {
    id: u32,
    curr_request: Option<Request>,
    body_buf: Vec<u8>,
    should_handle_request: bool,
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

    pub fn should_handle_request(&self) -> bool {
        match self {
            Stream::Idle(d) => d.should_handle_request,
            Stream::Open(d) => d.should_handle_request,
            Stream::Closed(d) => d.should_handle_request,
        }
    }

    pub fn set_request_handled(&mut self) {
        match self {
            Stream::Idle(d) => d.should_handle_request = false,
            Stream::Open(d) => d.should_handle_request = false,
            Stream::Closed(d) => d.should_handle_request = false,
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
            should_handle_request: false,
            body_buf: Vec::new(),
        })
    }
    pub fn on_frame(self, decoder: &mut Decoder, frame: Frame) -> Stream {
        println!("Stream {} processing frame: {:#?}", self.id(), frame);

        match self {
            Stream::Idle(data) => Stream::idle_on_frame_impl(frame, decoder, data),
            Stream::Open(data) => Stream::open_on_frame_impl(frame, decoder, data),
            _ => {
                unimplemented!();
            }
        }
    }

    pub fn send(&mut self, resp: Response, encoder: &mut Encoder, writer: &mut impl Write) {
        println!("Sending http response: {:#?}", resp);
        // Send Header Frame.
        {
            let h2headers = resp.h2headers();
            let mut flag_mask = h2::HeaderFlagMask(0);
            flag_mask.add(HeaderFlag::EndHeaders);

            println!("H2 headers to send: {:#?}", h2headers);

            let mut encodeable_headers: Vec<(&[u8], &[u8])> = Vec::new();
            for header in &h2headers {
                encodeable_headers.push((header.key.as_bytes(), header.value.as_bytes()));
            }
            let frame_body = encoder.encode(encodeable_headers);
            let frame_header = h2::Header::from_fields(
                frame_body.len() as u32,
                FrameType::Headers,
                flag_mask,
                self.id(),
            );

            let frame = Frame::new(frame_header, frame_body);
            let frame_buf = frame.serialize();

            println!("Writing header frame: {:#?}", frame);
            writer
                .write(&frame_buf)
                .expect("Failed to send header frame");
        }

        // Send data frames
        {
            let frame_body = resp.body;

            let mut flag_mask = h2::HeaderFlagMask(0);
            flag_mask.add(HeaderFlag::EndSream);
            let frame_header = h2::Header::from_fields(
                frame_body.len() as u32,
                FrameType::Data,
                flag_mask,
                self.id(),
            );

            let frame = Frame::new(frame_header, frame_body);
            let frame_buf = frame.serialize();
            writer
                .write(&frame_buf)
                .expect("Failed to send header frame");
        }
    }

    fn idle_on_frame_impl(frame: Frame, decoder: &mut Decoder, mut data: StreamData) -> Stream {
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

        if frame.header.flag_mask().check(HeaderFlag::EndSream) {
            data.should_handle_request = true;
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
            req = Request::empty();
        }
        req = Request::from_h2_headers(headers).expect("Invalid h2 headers");
        data.curr_request = Some(req);

        Stream::Open(data)
    }

    fn open_on_frame_impl(frame: Frame, decoder: &mut Decoder, mut data: StreamData) -> Stream {
        match frame.header.frame_type() {
            FrameType::Data => {}
            _ => {
                println!("Open stream state does not accept non data frames");
                return Stream::Idle(data);
            }
        }

        for b in frame.body {
            data.body_buf.push(b);
        }

        if frame.header.flag_mask().check(HeaderFlag::EndSream) {
            data.should_handle_request = true;
            let mut curr_req = data.curr_request.expect("Current request should not be empty while receiving data frame. Header should've been given");
            curr_req.set_body(data.body_buf);
            data.body_buf = Vec::new();
            data.curr_request = Some(curr_req);
        }

        Stream::Open(data)
    }
}
