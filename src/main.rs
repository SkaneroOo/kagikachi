mod sockets;
mod utils;

use std::collections::HashMap;

use sockets::{errors::SocketError, frame::DataFrame, response::Response, SocketServer, frame::Opcode};

#[derive(Debug, Clone)]
enum Value {
    String(String),
    Binary(Vec<u8>),
    Integer(isize),
    Boolean(bool),
    Float(f64),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Null
}

#[derive(Debug, Clone)]
enum ConversionError {
    InvalidType(String),
    InvalidValue(String)
}

fn look_for_closing(value: &str, opening: char) -> Option<usize> {
    let closing = match opening {
        '[' => ']',
        '{' => '}',
        '(' => ')',
        _ => panic!("Invalid opening character")
    };
    let mut depth = 0;
    let mut in_str = false;
    let mut prev = '\0';
    for (i, c) in value.chars().enumerate() {
        if c == '"' && prev != '\\' {   
            in_str = !in_str;
        }
        if c == opening && !in_str {
            depth += 1;
        }
        if c == closing && !in_str {
            depth -= 1;
        }
        if depth == 0 {
            return Some(i)
        }
        prev = c;
    }
    return None
}

fn look_for_comma(value: &str) -> Option<usize> {
    let mut in_str = false;
    let mut prev = '\0';
    for (i, c) in value.chars().enumerate() {
        if c == '"' && prev != '\\' {   
            in_str = !in_str;
        }
        if c == ',' && !in_str {
            return Some(i)
        }
        prev = c;
    }
    return None
}

impl TryFrom<String> for Value {
    type Error = ConversionError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim().to_string();
        if value.starts_with("\"") && value.ends_with("\"") {
            return Ok(Self::String(value[1..value.len() - 1].to_string()))
        }
        if value.starts_with("b\'") && value.ends_with("\'") {
            return Ok(Self::Binary(utils::decode(&value[2..value.len() - 1])))
        }
        if value.starts_with("[") && value.ends_with("]") {
            let mut values = vec![];

            let mut value = value[1..value.len() - 1].to_string();
            let mut split_at;

            loop {
                if value.starts_with("[") | value.starts_with("{") {
                    split_at = match look_for_closing(&value, value.chars().next().unwrap()) {
                        Some(i) => i,
                        None => return Err(ConversionError::InvalidValue(value))
                    };
                } else {
                    split_at = match look_for_comma(&value) {
                        Some(i) => i,
                        None => break
                    };
                }
                let temp = value[0..split_at].to_string();
                value = value[split_at + 1..].to_string();
                values.push(Value::try_from(temp.clone()).map_err(|_| ConversionError::InvalidValue(temp))?);
            }
            values.push(Value::try_from(value.clone()).map_err(|_| ConversionError::InvalidValue(value))?);
            
            return Ok(Self::Array(values))
        }
        if value.starts_with("{") && value.ends_with("}") {
            let mut object = HashMap::new();
            
            let mut value = value[1..value.len() - 1].to_string();

            while let Some((k, v)) = value.split_once(":") {
                let k = k.trim().to_owned();
                let mut v = v.trim().to_owned();
                if v.starts_with("[") | v.starts_with("{") {
                    let end = look_for_closing(&v, v.chars().next().unwrap()).unwrap();
                    if v.bytes().nth(end + 1) == Some(b',') {
                        value = v[end + 2..].to_string();
                    } else {
                        value = v[end + 1..].to_string();
                    }
                    v = v[..end + 1].to_owned();
                } else {
                    let (arg, rest) = match look_for_comma(&v) {
                        Some(i) => {
                            let (a, b) = v.split_at(i);
                            (a.to_string(), b[1..].to_string())
                        },
                        None => (v, "".to_string())
                    };
                    v = arg;
                    value = rest;
                }
                object.insert(k[1..k.len() - 1].to_string(), Value::try_from(v.clone())?);
            }
            return Ok(Self::Object(object))
        }
        if value.contains(".") { 
            if let Ok(value) = value.parse::<f64>() {
                return Ok(Self::Float(value))
            }
        }
        if let Ok(value) = value.parse::<isize>() {
            return Ok(Self::Integer(value))
        }
        if let Ok(value) = value.parse::<bool>() {
            return Ok(Self::Boolean(value))
        }
        if value == "null" {
            return Ok(Self::Null)
        }
        
        Err(ConversionError::InvalidType(value))
    }
}

fn message_handler(msg: DataFrame, storage: &mut HashMap<String, Value>) -> Response {
    assert!(msg.opcode == Opcode::Text);
    let message = msg.payload.string().expect("Assertion failed, check if payload was properly decoded");
    let (command, args) = match message.split_once(" ") {
        Some((command, args)) => (command, args),
        None => (message.as_str(), "")
    };

    match command {
        "set" => {
            let (key, value) = match args.split_once(" ") {
                Some((key, value)) => (key, value),
                None => return Response::builder().set_body("Invalid arguments")
            };
            storage.insert(key.to_string(), match Value::try_from(value.to_string()) {
                Ok(value) => value,
                Err(e) => return Response::builder().set_body(format!("Invalid value: {e:?}"))
            });
            Response::builder().set_body("OK")
        },
        "get" => {
            let (key, path) = match args.split_once(".") {
                Some((key, path)) => (key, Some(path)),
                None => (args, None)
            };
            let value = match storage.get(key) {
                Some(value) => match path {
                    None => value.clone(),
                    Some(path) => {
                        let mut value = value.clone();
                        for p in path.split(".") {
                            match value {
                                Value::Object(object) => value = match object.get(p) {
                                    Some(value) => value.clone(),
                                    None => return Response::builder().set_body("Key not found")
                                },
                                _ => return Response::builder().set_body("Invalid path")
                            }
                        }
                        value
                    }
                },
                None => return Response::builder().set_body("Key not found")
            };
            Response::builder().set_body(format!("{value:?}"))
        },
        _ => Response::builder().set_body("Unknown command")
    }
}

fn error_handler(e: SocketError) {
    println!("Error: {e}")
}

fn main() {
    let cache: HashMap<String, Value> = HashMap::new();
    let server = SocketServer::new(message_handler, error_handler, cache);
    server.run();
}
