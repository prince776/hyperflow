use hpack::{Decoder, Encoder};

use crate::h2::{Frame, Header, HeaderFlag};

pub struct StreamData {
    id: u32,
}

pub enum Stream {
    Idle(StreamData),
    Open(StreamData),
    Closed(StreamData),
}

impl Stream {
    fn id(&self) -> u32 {
        match self {
            Stream::Idle(d) => d.id,
            Stream::Open(d) => d.id,
            Stream::Closed(d) => d.id,
        }
    }
}

impl Stream {
    pub fn new(id: u32) -> Stream {
        println!("Started new idle stream");
        Stream::Idle(StreamData { id })
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

    fn on_frame_idle(frame: Frame, decoder: &mut Decoder, data: StreamData) -> Stream {
        if !frame.header.flag_mask().check(HeaderFlag::EndHeaders) {
            unimplemented!("continuation header not supported");
        }

        let decoded_headers = decoder
            .decode(&frame.body)
            .expect("Failed to decode headers");

        println!("Got Headers in stream: {}", data.id);
        for (key, value) in decoded_headers {
            println!(
                "{}:{}",
                String::from_utf8_lossy(&key),
                String::from_utf8_lossy(&value)
            );
        }

        Stream::Idle(data)
    }
}
