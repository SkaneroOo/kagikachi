use std::{collections::HashMap, io::{BufRead as _, BufReader, Write as _}, net::TcpStream};

use utils::{sha1, encode};

use super::errors::SocketError;

pub fn handle_handshake(mut conn: &TcpStream) -> Result<(), SocketError> {
    let buff = BufReader::new(&mut conn);
    let lines: Vec<_> = buff.lines().map(|res| res.unwrap()).take_while(|line| !line.is_empty()).collect();
    // println!("Request: {:?}", lines);

    let mut headers: HashMap<_, _> = HashMap::new();

    for line in lines.iter().skip(1) {
        let (key, value) = line.split_once(": ").unwrap();
        headers.insert(key, value);
    }

    if !headers.get("Connection").unwrap().contains("Upgrade") {
        let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
        conn.write_all(response.as_bytes()).unwrap();
        return Err(SocketError::InvalidHandshake)
    }

    if !headers.get("Upgrade").unwrap().eq_ignore_ascii_case("websocket") {
        let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
        conn.write_all(response.as_bytes()).unwrap();
        return Err(SocketError::InvalidHandshake)
    }

    let key = headers.get("Sec-WebSocket-Key").unwrap();
    let guid = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    let hash = sha1(format!("{key}{guid}").as_bytes());
    let resp = encode(&hash);

    let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {resp}\r\n\r\n");
    conn.write_all(response.as_bytes()).unwrap();
    conn.flush().unwrap();

    Ok(())
}