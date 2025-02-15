mod sockets;
mod utils;

use sockets::{errors::SocketError, frame::DataFrame, response::Response, SocketServer};

fn message_handler(msg: DataFrame, _: &()) -> Response {
    Response::builder().set_body(msg.payload)
}

fn error_handler(e: SocketError) {
    println!("Error: {e}")
}

fn main() {
    let server = SocketServer::new(message_handler, error_handler, ());
    server.run();
}
