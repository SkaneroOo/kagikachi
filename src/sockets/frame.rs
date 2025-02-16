use std::{io::{Read as _, Write as _}, net::TcpStream, ops::BitOr};

use crate::sockets::response::Response;

use super::errors;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Opcode {
    Continuation = 0,
    Text = 1,
    Binary = 2,
    ConnectionClosed = 8,
    Ping = 9,
    Pong = 10
}

impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        match value {
            0 => Opcode::Continuation,
            1 => Opcode::Text,
            2 => Opcode::Binary,
            8 => Opcode::ConnectionClosed,
            9 => Opcode::Ping,
            10 => Opcode::Pong,
            _ => panic!("Opcode not present in the standard")
        }
    }
}

impl From<Opcode> for u8 {
    fn from(value: Opcode) -> Self {
        match value {
            Opcode::Continuation => 0,
            Opcode::Text => 1,
            Opcode::Binary => 2,
            Opcode::ConnectionClosed => 8,
            Opcode::Ping => 9,
            Opcode::Pong => 10
        }
    }
}

impl BitOr for Opcode {
    type Output = u8;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as u8 | rhs as u8
    }
}

impl BitOr<Opcode> for u8 {
    type Output = u8;

    fn bitor(self, rhs: Opcode) -> Self::Output {
        self | rhs as u8
    }
}

#[derive(Debug, Clone)]
pub enum Payload {
    Text(String),
    Binary(Vec<u8>)
}

impl Payload {
    #[allow(unused)]
    pub fn string(self) -> Option<String> {
        match self {
            Payload::Text(s) => Some(s),
            _ => None
        }
    }

    #[allow(unused)]
    pub fn bytes(self) -> Vec<u8> {
        match self {
            Payload::Text(s) => s.into_bytes(),
            Payload::Binary(b) => b
        }
    }
}

impl From<String> for Payload {
    fn from(value: String) -> Self {
        Payload::Text(value)
    }
}

impl From<Vec<u8>> for Payload {
    fn from(value: Vec<u8>) -> Self {
        Payload::Binary(value)
    }
}

impl From<&str> for Payload {
    fn from(value: &str) -> Self {
        Payload::Text(value.to_string())
    }
}

impl Into<Vec<u8>> for Payload {
    fn into(self) -> Vec<u8> {
        match self {
            Payload::Text(s) => s.into_bytes(),
            Payload::Binary(b) => b
        }
    }
}

#[derive(Debug)]
pub struct DataFrame {
    #[allow(unused)]
    pub flags: u8,
    pub opcode: Opcode,
    #[allow(unused)]
    pub length: usize,
    pub mask: Option<[u8; 4]>,
    pub payload: Payload
}

impl DataFrame {
    pub fn pong(&self, conn: &mut TcpStream) -> Result<(), errors::SocketError> {
        assert!(self.opcode == Opcode::Ping);
        conn.write(&Response::pong(self)).map_err(|_| errors::SocketError::ConnectionClosed)?;
        Ok(())
    }
}

pub trait ReadDataFrame {
    fn read_frame(&mut self) -> Result<DataFrame, errors::SocketError>;
}

impl ReadDataFrame for TcpStream {
    fn read_frame(&mut self) -> Result<DataFrame, errors::SocketError> {

        let mut buff = [0; 2];
        if let Err(_) = self.read(&mut buff) {
            return Err(errors::SocketError::CannotReadPayload);
        }
        let flags = buff[0] & 0xf0;
        let opcode = (buff[0] & 0x0f).into();
        let is_mask = buff[1] & 0b1000_0000 != 0;
        let length = match buff[1] & 0b0111_1111 {
            126 => {
                let mut buff = [0; 2];
                if let Err(_) = self.read(&mut buff) {
                    return Err(errors::SocketError::CannotReadPayload);
                }
                u16::from_be_bytes(buff) as usize
            },
            127 => {
                let mut buff = [0; 8];
                if let Err(_) = self.read(&mut buff) {
                    return Err(errors::SocketError::CannotReadPayload);
                }
                usize::from_be_bytes(buff)
            },
            _ => (buff[1] & 0b0111_1111) as usize
        };

        let mut mask = [0; 4];

        if is_mask {
            if let Err(_) = self.read(&mut mask) {
                return Err(errors::SocketError::CannotReadPayload);
            }
        }

        let mut payload = vec![0; length];
        match self.read(&mut payload) {
            Ok(i) => {
                if i != length {
                    return Err(errors::SocketError::InvalidFrame);
                }
            },
            Err(_) => return Err(errors::SocketError::CannotReadPayload)
        }

        if is_mask {
            for i in 0..length {
                payload[i] = payload[i] ^ mask[i % 4];
            }
        }

        Ok(DataFrame {
            flags,
            opcode,
            length,
            mask: if is_mask { Some(mask) } else { None },
            payload: if opcode == Opcode::Text { Payload::Text(String::from_utf8(payload).unwrap()) } else { Payload::Binary(payload) }
        })
    }
}