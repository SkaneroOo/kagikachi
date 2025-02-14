mod sockets;
mod utils;

use sockets::{frame::{Opcode, ReadDataFrame}, handshake::handle_handshake, response::Response};
use utils::Rand;

use std::{
    io::Write, net::{Shutdown, TcpListener, TcpStream}, sync::Arc, thread
};


fn handle_connection(mut conn: TcpStream, rand: Arc<Rand>) {
    
    match handle_handshake(&conn) {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {e}");
            return;
        }
    }

    loop {

        let data = match conn.read_frame() {
            Ok(data) => data,
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        };

        if data.opcode == Opcode::Ping {
            data.pong(&mut conn).unwrap();
            continue;
        }

        if data.opcode == Opcode::ConnectionClosed {
            conn.shutdown(Shutdown::Both).unwrap();
            break;
        }

        let resp_mask = rand.get_mask();

        let response = Response::builder()
            .set_body(data.payload)
            .set_mask(resp_mask)
            .build();
        
        conn.write_all(&response).unwrap();
        conn.flush().unwrap();
        
    }
}


fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let rand = Arc::new(Rand::new());

    thread::scope(|s| {
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let rand = Arc::clone(&rand);
            s.spawn(move || {
                handle_connection(stream, rand);
            });
        }
    });
}
