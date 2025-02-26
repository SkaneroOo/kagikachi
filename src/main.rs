mod sockets;
mod utils;

use utils::json::Value;

use std::collections::HashMap;

use sockets::{errors::SocketError, frame::DataFrame, response::Response, SocketServer, frame::Opcode};


fn set_cmd(args: &str, storage: &mut HashMap<String, Value>) -> Response {
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
}

fn get_cmd(args: &str, storage: &mut HashMap<String, Value>) -> Response {
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
}

fn del_cmd(args: &str, storage: &mut HashMap<String, Value>) -> Response {
    let (key, path) = match args.split_once(".") {
        Some((key, path)) => (key, Some(path)),
        None => (args, None)
    };
    match path {
        None => {
            storage.remove(key);
            Response::builder().set_body("OK")
        }
        Some(path) => {
            match storage.get_mut(key) {
                Some(val) => {
                    let (parent, key) = match path.rsplit_once(".") {
                        Some((parent, key)) => (parent, key),
                        None => {
                            match val {
                                Value::Object(map) => {
                                    match map.remove(key) {
                                        Some(_) => return Response::builder().set_body("OK"),
                                        None => return Response::builder().set_body("Key not found")
                                    }
                                },
                                Value::Array(arr) => {
                                    let index = match path.parse::<usize>() {
                                        Ok(i) => i,
                                        Err(_) => return Response::builder().set_body("Invalid index")
                                    };
                                    if index >= arr.len() {
                                        return Response::builder().set_body("Index out of range")
                                    }
                                    arr.remove(index);
                                    return Response::builder().set_body("OK")
                                },
                                _ => return Response::builder().set_body("Invalid type")
                            }
                        }
                    };
                    match val.get_mut_element(parent) {
                        Ok(val) => match val {
                            Value::Object(map) => {
                                match map.remove(key) {
                                    Some(_) => Response::builder().set_body("OK"),
                                    None => Response::builder().set_body("Key not found")
                                }
                            },
                            Value::Array(arr) => {
                                let index = match key.parse::<usize>() {
                                    Ok(i) => i,
                                    Err(_) => return Response::builder().set_body("Invalid index")
                                };
                                arr.remove(index);
                                Response::builder().set_body("OK")
                            },
                            _ => Response::builder().set_body("Invalid type")
                        },
                        Err(e) => Response::builder().set_body(e)
                    }
                },
                None => {
                    Response::builder().set_body("Key not found")
                }
            }
        }
    }
}

fn dump_cmd(storage: &HashMap<String, Value>) -> Response {
    let value = Value::Object(storage.clone());
    Response::builder().set_body(Into::<String>::into(value))
}

fn load_cmd(args: &str, storage: &mut HashMap<String, Value>) -> Response {
    let value = match Value::try_from(args.to_string()) {
        Ok(v) => v,
        Err(e) => return Response::builder().set_body(format!("Invalid value: {e:?}"))
    };
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                storage.insert(key, value);
            }
            Response::builder().set_body("OK")
        },
        _ => Response::builder().set_body("Invalid type")
    }
}

fn message_handler(msg: DataFrame, storage: &mut HashMap<String, Value>) -> Response {
    match msg.opcode {
        Opcode::Text => (),
        _ => return Response::builder().set_body("Invalid message type")
    }
    let message = msg.payload.string().expect("Assertion failed, check if payload was properly decoded");
    let (command, args) = match message.split_once(" ") {
        Some((command, args)) => (command, args),
        None => (message.as_str(), "")
    };

    match command.to_ascii_lowercase().as_str() {
        "set" => set_cmd(args, storage),
        "get" => get_cmd(args, storage),
        "del" => del_cmd(args, storage),
        "dump" => dump_cmd(storage),
        "load" => load_cmd(args, storage),
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
