mod utils;

use utils::{Rand, sha1, encode};

use std::{
    collections::HashMap, io::{BufRead, BufReader, Read, Write}, net::{TcpListener, TcpStream}, sync::Arc, thread
};


fn handle_connection(mut conn: TcpStream, rand: Arc<Rand>) {
    println!("Connection established");
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
        return
    }

    if !headers.get("Upgrade").unwrap().eq_ignore_ascii_case("websocket") {
        let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
        conn.write_all(response.as_bytes()).unwrap();
        return
    }

    let key = headers.get("Sec-WebSocket-Key").unwrap();
    let guid = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    println!("{key}{guid}");
    let hash = sha1(format!("{key}{guid}").as_bytes());
    for i in 0..hash.len() {
        print!("{:02x}", hash[i]);
    }
    println!();
    let resp = encode(&hash);
    println!("{resp}");

    let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {resp}\r\n\r\n");
    conn.write_all(response.as_bytes()).unwrap();
    conn.flush().unwrap();
    // conn.set_nonblocking(true).unwrap();

    loop {
        let mut buff = [0; 2];
        if let Ok(i) = conn.read(&mut buff) {
            if i == 0 {
                continue;
            }
            let mut flags = buff[0];
            let is_mask = buff[1] & 0b1000_0000 != 0;
            let length = match buff[1] & 0b0111_1111 {
                126 => {
                    let mut buff = [0; 2];
                    conn.read(&mut buff).unwrap();
                    u16::from_be_bytes(buff) as usize
                },
                127 => {
                    let mut buff = [0; 8];
                    conn.read(&mut buff).unwrap();
                    usize::from_be_bytes(buff)
                },
                _ => (buff[1] & 0b0111_1111) as usize
            };

            let mut mask = [0; 4];
            if is_mask {
                conn.read(&mut mask).unwrap();
            }

            let mut data = vec![0; length];
            conn.read(&mut data).unwrap();

            for i in 0..length {
                data[i] = data[i] ^ mask[i % 4];
            }

            // let mut dump = [0; 1024];
            // conn.read(&mut dump).unwrap();
            conn.flush().unwrap();

            println!("Request: {:?}", data);

            let resp_mask = rand.get_mask();
            println!("Resp_mask: {resp_mask:?}");
            if flags & 0x09 == 0x09 {
                flags += 1;
            }

            let mut response = vec![];
            response.push(flags);
            match length {
                0..=125 => response.push(length as u8),
                126..=65535 => {
                    response.push(126);
                    response.extend_from_slice(&(length as u16).to_be_bytes());
                },
                _ => {
                    response.push(127);
                    response.extend_from_slice(&(length as u64).to_be_bytes());
                }
            }
            response[1] |= 0b1000_0000;
            response.extend_from_slice(&resp_mask);
            for i in 0..length {
                response.push(data[i] ^ resp_mask[i % 4]);
            }
            println!("Response: {response:?}");
            conn.write_all(&response).unwrap();
            conn.flush().unwrap();

        }
        
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
