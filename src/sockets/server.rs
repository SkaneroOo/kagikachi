use std::{
    io::Write, net::{Shutdown, TcpListener, TcpStream}, sync::Mutex, thread
};
use crate::{utils::Rand, sockets::{errors::SocketError, frame::{DataFrame, Opcode, ReadDataFrame}, handshake::handle_handshake, response::Response}};


pub struct SocketServer<T> where T: Send{
    listener: TcpListener,
    rand: Rand,
    message_handler: fn(DataFrame, &T) -> Response,
    error_handler: fn(SocketError),
    internal_data: Mutex<T>
}

impl<T> SocketServer<T> where T: Send{
    pub fn new(message_handler: fn(DataFrame, &T) -> Response, error_handler: fn(SocketError), internal_data: T) -> Self {
        Self {
            listener: TcpListener::bind("localhost:7878").unwrap(),
            rand: Rand::new(),
            message_handler,
            error_handler,
            internal_data: Mutex::new(internal_data)
        }
    }

    pub fn run(&self) {
        thread::scope(|s| {
            for stream in self.listener.incoming() {
                match stream {
                    Ok(conn) => {
                        match handle_handshake(&conn) {
                            Ok(()) => {},
                            Err(e) => {
                                (self.error_handler)(e);
                                continue;
                            }
                        }
                        s.spawn(move || {
                            self.main_loop(conn);
                        });
                    },
                    Err(_e) => {
                        (self.error_handler)(SocketError::UnknownError)
                    }
                }
            }
        })
    }

    fn main_loop(&self, mut conn: TcpStream) {
        loop {
            let data = match conn.read_frame() {
                Ok(data) => data,
                Err(e) => {
                    (self.error_handler)(e);
                    continue;
                }
            };

            if data.opcode == Opcode::Ping {
                data.pong(&mut conn).unwrap();
                continue;
            }

            if data.opcode == Opcode::ConnectionClosed {
                conn.shutdown(Shutdown::Both).unwrap();
                return
            }

            let response;
            {
                let lock = self.internal_data.lock().unwrap();
                response = (self.message_handler)(data, &lock);
            }

            let payload = response.set_mask(self.rand.get_mask()).build();
            conn.write_all(&payload).unwrap();
            conn.flush().unwrap();
        }
    }
}
