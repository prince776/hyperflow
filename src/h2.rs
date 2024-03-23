use std::{
    any::Any,
    borrow::Cow,
    mem::{self, discriminant},
};

#[derive(Debug)]
pub struct Frame {
    pub header: Header,
    pub body: Vec<u8>,
}

impl Frame {
    pub fn new(header: Header, body: Vec<u8>) -> Frame {
        Frame { header, body }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut res = Vec::new();

        let buf = self.header.length.to_be_bytes();
        res.push(buf[1]);
        res.push(buf[2]);
        res.push(buf[3]);

        let buf = self.header.frame_type.val().to_be_bytes();
        res.push(buf[0]);

        let buf = self.header.flag_mask.0.to_be_bytes();
        res.push(buf[0]);

        let buf = self.header.stream_identifier.to_be_bytes();
        res.push(buf[0]);
        res.push(buf[1]);
        res.push(buf[2]);
        res.push(buf[3]);

        for b in self.body.iter() {
            res.push(*b);
        }

        res
    }
}

#[derive(Debug)]
pub struct Header {
    length: u32,
    frame_type: FrameType,
    flag_mask: HeaderFlagMask,
    stream_identifier: u32,
}

impl Header {
    pub fn from_fields(
        length: u32,
        frame_type: FrameType,
        flag_mask: HeaderFlagMask,
        stream_id: u32,
    ) -> Header {
        Header {
            length,
            frame_type,
            flag_mask,
            stream_identifier: stream_id,
        }
    }
}

#[derive(Clone, Debug)]
pub enum FrameType {
    Data,
    Headers,
    Priority,
    RstStream,
    Settings,
    PushPromise,
    Ping,
    GoAway,
    WindowUpdate,
    Continuation,
}

impl FrameType {
    pub fn new<'a>(val: u8) -> Result<FrameType, &'a str> {
        match val {
            0 => Ok(FrameType::Data),
            1 => Ok(FrameType::Headers),
            2 => Ok(FrameType::Priority),
            3 => Ok(FrameType::RstStream),
            4 => Ok(FrameType::Settings),
            5 => Ok(FrameType::PushPromise),
            6 => Ok(FrameType::Ping),
            7 => Ok(FrameType::GoAway),
            8 => Ok(FrameType::WindowUpdate),
            9 => Ok(FrameType::Continuation),
            _ => Err("not a valid frame type"),
        }
    }

    pub fn val<'a>(&self) -> u8 {
        match self {
            FrameType::Data => 0,
            FrameType::Headers => 1,
            FrameType::Priority => 2,
            FrameType::RstStream => 3,
            FrameType::Settings => 4,
            FrameType::PushPromise => 5,
            FrameType::Ping => 6,
            FrameType::GoAway => 7,
            FrameType::WindowUpdate => 8,
            FrameType::Continuation => 9,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum HeaderFlag {
    EndSream = 0x1,
    EndHeaders = 0x4,
    Padded = 0x8,
    Priority = 0x20,
}

#[derive(Clone, Debug)]
pub struct HeaderFlagMask(pub u8);

impl HeaderFlagMask {
    pub fn check(&self, flag: HeaderFlag) -> bool {
        let masked = self.0 & (flag as u8);
        return masked > 0;
    }

    pub fn add(&mut self, flag: HeaderFlag) {
        self.0 |= flag as u8;
    }

    pub fn remove(&mut self, flag: HeaderFlag) {
        self.0 &= !(flag as u8);
    }
}

impl Header {
    pub fn new<'a>(buf: &[u8]) -> Result<Header, &'a str> {
        let length = u32::from_be_bytes([0x00, buf[0], buf[1], buf[2]]);

        let frametype_val = u8::from_be_bytes([buf[3]]);
        let frame_type = FrameType::new(frametype_val)?;

        let flag_mask = HeaderFlagMask(u8::from_be_bytes([buf[4]]));

        let mut stream_identifier = u32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]);
        stream_identifier %= 1 << 31;

        Ok(Header {
            length,
            frame_type,
            flag_mask,
            stream_identifier,
        })
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn frame_type(&self) -> FrameType {
        self.frame_type.clone()
    }

    pub fn flag_mask(&self) -> HeaderFlagMask {
        self.flag_mask.clone()
    }

    pub fn stream_identifier(&self) -> u32 {
        self.stream_identifier
    }
}
