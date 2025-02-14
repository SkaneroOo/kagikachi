use std::fmt::Display;

#[derive(Debug)]
pub enum SocketError {
    CannotReadPayload,
    ConnectionClosed,
    InvalidHandshake,
    InvalidFrame,
    #[allow(unused)]
    UnknownError
}

impl Display for SocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SocketError::CannotReadPayload => write!(f, "Cannot read payload"),
            SocketError::ConnectionClosed => write!(f, "Connection closed"),
            SocketError::InvalidHandshake => write!(f, "Invalid handshake"),
            SocketError::InvalidFrame => write!(f, "Invalid frame"),
            SocketError::UnknownError => write!(f, "Unknown error")
        }
    }
}