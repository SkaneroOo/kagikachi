use super::frame::{DataFrame, Opcode, Payload};

pub struct Response {
    pub flags: u8,
    pub opcode: Opcode,
    pub length: usize,
    pub mask: Option<[u8; 4]>,
    pub payload: Payload
}

impl Response {
    pub fn builder() -> Self {
        Response {
            flags: 0x80,
            opcode: Opcode::Binary,
            length: 0,
            mask: None,
            payload: Payload::Binary(vec![])
        }
    }

    pub fn set_body(mut self, payload: impl Into<Payload>) -> Self {
        self.payload = payload.into();
        match &self.payload {
            Payload::Text(s) => {
                self.length = s.bytes().len();
                self.opcode = Opcode::Text;
            },
            Payload::Binary(b) => {
                self.length = b.len();
                self.opcode = Opcode::Binary;
            }
        }
        self
    }

    pub fn set_mask(mut self, mask: [u8; 4]) -> Self {
        self.mask = Some(mask);
        self
    }

    pub fn build(self) -> Vec<u8> {
        let mut buff: Vec<u8> = Vec::new();
        buff.push(self.flags | self.opcode);
        match self.length {
            0..=125 => buff.push(self.length as u8 | if self.mask.is_some() { 0x80 } else { 0 }),
            126..=65535 => {
                buff.push(126 | if self.mask.is_some() { 0x80 } else { 0 });
                buff.extend_from_slice(&(self.length as u16).to_be_bytes());
            },
            _ => {
                buff.push(127 | if self.mask.is_some() { 0x80 } else { 0 });
                buff.extend_from_slice(&(self.length as u64).to_be_bytes());
            }
        }

        if let Some(mask) = self.mask {
            buff.extend_from_slice(&mask);
        }

        let mut payload: Vec<u8> = self.payload.into();

        if let Some(mask) = self.mask {
            for (i, byte) in payload.iter_mut().enumerate() {
                *byte ^= mask[i % 4];
            }
        }

        buff.extend(payload);
        buff
    }

    pub fn pong(ping: &DataFrame)-> Vec<u8> {
        let mut response = Response::builder()
            .set_body(ping.payload.clone())
            .set_mask(ping.mask.unwrap());
        response.opcode = Opcode::Pong;
        response.build()
    }
}