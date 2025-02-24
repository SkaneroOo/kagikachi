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

impl Value {
    pub fn get_element(&self, path: &str) -> Result<&Value, String> {
        if path.is_empty() {
            return Ok(self);
        }
        let (key, path) = match path.split_once(".") {
            Some((key, path)) => (key, path),
            None => (path, "")
        };
        match self {
            Value::Object(map) => match map.get(key) {
                Some(value) => value.get_element(path),
                None => Err(format!("Key {} not found", key))
            },
            Value::Array(arr) => match key.parse::<usize>() {
                Ok(i) => match arr.get(i) {
                    Some(value) => value.get_element(path),
                    None => Err(format!("Index {} out of range", i))
                },
                Err(_) => Err(format!("{} is not a valid index for array", key))
            }
            _ => Err(format!("Invalid type: expected Object, got {}", self.typename()))
        }
    }

    pub fn get_mut_element(&mut self, path: &str) -> Result<&mut Value, String> {
        if path.is_empty() {
            return Ok(self);
        }
        let (key, path) = match path.split_once(".") {
            Some((key, path)) => (key, path),
            None => (path, "")
        };
        match self {
            Value::Object(map) => match map.get_mut(key) {
                Some(value) => value.get_mut_element(path),
                None => Err(format!("Key {} not found", key))
            },
            Value::Array(arr) => match key.parse::<usize>() {
                Ok(i) => match arr.get_mut(i) {
                    Some(value) => value.get_mut_element(path),
                    None => Err(format!("Index {} out of range", i))
                },
                Err(_) => Err(format!("{} is not a valid index for array", key))
            }
            _ => Err(format!("Invalid type: expected Object, got {}", self.typename()))
        }
    }

    fn typename(&self) -> String {
        match self {
            Value::String(_) => "String",
            Value::Binary(_) => "Binary",
            Value::Integer(_) => "Integer",
            Value::Boolean(_) => "Boolean",
            Value::Float(_) => "Float",
            Value::Array(_) => "Array",
            Value::Object(_) => "Object",
            Value::Null => "Null"
        }.to_string()
    }
}

#[derive(Debug, Clone)]
enum ConversionError {
    InvalidType(String),
    InvalidValue(String)
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::InvalidType(s) => write!(f, "Invalid type: {}", s),
            ConversionError::InvalidValue(s) => write!(f, "Invalid value: {}", s)
        }
    }
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

fn look_for_string_end(value: &str) -> Option<usize> {
    let mut prev = '"';
    for (i, c) in value.chars().enumerate().skip(1) {
        if c == '"' && prev != '\\' {   
            return Some(i)
        }
        prev = c;
    }
    return None
}

fn look_for_char(value: &str, ch: char) -> Option<usize> {
    let mut in_str = false;
    let mut prev = '\0';
    let mut skip = 0;
    let mut iter = value.chars().enumerate();
    for (i, c) in &mut iter {
        if skip > 0 {
            skip -= 1;
            continue
        }
        if c == '"' && prev != '\\' {   
            in_str = !in_str;
        }
        if c == ch && !in_str {
            return Some(i)
        }
        if (c == '{' || c == '[' || c == '(') && (!in_str && prev != '\\') {
            skip = match look_for_closing(&value[i..], c) {
                Some(i) => i,
                None => return None
            };
        }
        prev = c;
    }
    return None
}

impl Into<String> for Value {
    fn into(self) -> String {
        Into::<String>::into(&self)
    }
}

impl Into<String> for &Value {
    fn into(self) -> String {
        match self {
            Value::String(s) => format!("\"{}\"", s),
            Value::Binary(v) => format!("b'{}'", utils::encode(&v)),
            Value::Integer(i) => i.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Array(a) => {
                let mut s = String::from("[");
                let len = a.len();
                for (i, v) in a.into_iter().enumerate() {
                    s.push_str(&Into::<String>::into(v));
                    if i != len - 1 {
                        s.push_str(", ");
                    }
                }
                s.push_str("]");
                s
            },
            Value::Object(o) => {
                let mut s = String::from("{");
                let len = o.len();
                for (i, (k, v)) in o.into_iter().enumerate() {
                    s.push('"');
                    s.push_str(&k);
                    s.push('"');
                    s.push_str(": ");
                    s.push_str(&Into::<String>::into(v));
                    if i != len - 1 {
                        s.push_str(", ");
                    }
                }
                s.push_str("}");
                s
            },
            Value::Null => "null".to_string()
        }
    }
}

impl TryFrom<String> for Value {
    type Error = ConversionError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim().to_string();
        if value.starts_with("\"") && value.ends_with("\"") {
            if let Some(i) = look_for_string_end(&value) {
                if i != value.chars().count() - 1 {
                    println!("{} - {}", i, value.chars().count());
                    return Err(ConversionError::InvalidType(value))
                }
            }
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
                    split_at = match look_for_char(&value, ',') {
                        Some(i) => i,
                        None => break
                    };
                }
                let temp = value[0..split_at].to_string();
                value = value[split_at + 1..].to_string();
                values.push(Value::try_from(temp.clone())?);
            }
            values.push(Value::try_from(value.clone())?);
            
            return Ok(Self::Array(values))
        }
        if value.starts_with("{") && value.ends_with("}") {
            let mut object = HashMap::new();
            
            let mut value = value[1..value.len() - 1].to_string();

            while let Some(pos) = look_for_char(&value, ':') {
                let (k, v) = value.split_at(pos);
                let k = k.trim().to_owned();
                let mut v = v[1..].trim().to_owned();
                if v.starts_with("[") | v.starts_with("{") {
                    let end = look_for_closing(&v, v.chars().next().unwrap()).unwrap();
                    if v.bytes().nth(end + 1) == Some(b',') {
                        value = v[end + 2..].to_string();
                    } else {
                        value = v[end + 1..].to_string();
                    }
                    v = v[..end + 1].to_owned();
                } else {
                    let (arg, rest) = match look_for_char(&v, ',') {
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
            let value = match Value::try_from(value.to_string()) {
                Ok(v) => v,
                Err(e) => return Response::builder().set_body(format!("Invalid value: {e:?}"))
            };
            let (key, path) = match key.split_once(".") {
                Some((key, path)) => (key, Some(path)),
                None => (key, None)
            };
            match path {
                None => {
                    storage.insert(key.to_string(), value);
                    Response::builder().set_body("OK")
                },
                Some(path) => {
                    match storage.get_mut(key) {
                        Some(val) => match val.get_mut_element(path) {
                            Ok(val) => {
                                *val = value;
                                Response::builder().set_body("OK")
                            },
                            Err(e) => Response::builder().set_body(e)
                        },
                        None => Response::builder().set_body("Key not found")
                    }
                }
            }
        },
        "get" => {
            let (key, path) = match args.split_once(".") {
                Some((key, path)) => (key, Some(path)),
                None => (args, None)
            };
            let value = match storage.get(key) {
                Some(value) => match path {
                    None => value,
                    Some(path) => match value.get_element(path) {
                        Ok(value) => value,
                        Err(e) => return Response::builder().set_body(e)
                    }
                },
                None => return Response::builder().set_body("Key not found")
            };
            Response::builder().set_body(Into::<String>::into(value))
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
