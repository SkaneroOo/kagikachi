use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    String(String),
    Integer(isize),
    Float(f64),
    Boolean(bool),
    Null
}

impl Value {
    pub fn string(&self) -> Result<&String, &'static str> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err("Invalid type")
        }
    }

    pub fn array(&self) -> Result<&Vec<Value>, &'static str> {
        match self {
            Value::Array(a) => Ok(a),
            _ => Err("Invalid type")
        }
    }

    pub fn object(&self) -> Result<&HashMap<String, Value>, &'static str> {
        match self {
            Value::Object(o) => Ok(o),
            _ => Err("Invalid type")
        }
    }

    pub fn float(&self) -> Result<f64, &'static str> {
        match self {
            Value::Float(f) => Ok(*f),
            _ => Err("Invalid type")
        }
    }

    pub fn integer(&self) -> Result<isize, &'static str> {
        match self {
            Value::Integer(i) => Ok(*i),
            _ => Err("Invalid type")
        }
    }


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
            // Value::Binary(_) => "Binary",
            Value::Integer(_) => "Integer",
            Value::Boolean(_) => "Boolean",
            Value::Float(_) => "Float",
            Value::Array(_) => "Array",
            Value::Object(_) => "Object",
            Value::Null => "Null"
        }.to_string()
    }
}

impl Value {
    pub fn deserialize(value: &str) -> Result<Value, &'static str> {
        if value.is_empty() {
            return Ok(Value::Null)
        }
        let bytes = value.as_bytes();
        let mut position: usize = 0;
        skip_whitespace(bytes, &mut position);

        parse_value(bytes, &mut position)
    }

    pub fn serialize(&self) -> String {
        self.into()
    }
}

fn parse_value(bytes: &[u8], position: &mut usize) -> Result<Value, &'static str> {
    match bytes[*position] {
        b'{' => {
            return parse_object(bytes, position)
        }
        b'[' => {
            return parse_array(bytes, position)
        }
        b'"' => {
            return parse_string(bytes, position)
        }
        b't' | b'f' => {
            return parse_boolean(bytes, position)
        }
        b'n' => {
            return parse_null(bytes, position)
        }
        b'-' | b'0'..=b'9' => {
            return parse_number(bytes, position)
        }
        _ => {
            return Err("Invalid value")
        }
    }
}

fn skip_whitespace(bytes: &[u8], position: &mut usize) {
    for i in *position..bytes.len() {
        if !bytes[i].is_ascii_whitespace() {
            *position = i;
            return
        }
    }
    *position = bytes.len()
}

fn parse_object(bytes: &[u8], position: &mut usize) -> Result<Value, &'static str> {
    let mut map = HashMap::new();
    *position += 1;
    skip_whitespace(bytes, position);
    if *position == bytes.len() {
        return Err("Invalid data")
    }
    if bytes[*position] == b'}' {
        *position += 1;
        return Ok(Value::Object(map))
    }
    loop {
        let key = match parse_string(bytes, position) {
            Ok(v) => v.string().expect("Unexpected key type recieved").to_owned(),
            Err(_) => return Err("Invalid key")
        };

        skip_whitespace(bytes, position);
        if *position == bytes.len() {
            return Err("Invalid data")
        }
        if bytes[*position] != b':' {
            return Err("Invalid token")
        }
        *position += 1;
        skip_whitespace(bytes, position);
        if *position == bytes.len() {
            return Err("Invalid data")
        }

        let value = parse_value(bytes, position)?;

        map.insert(key, value);

        skip_whitespace(bytes, position);
        if *position == bytes.len() {
            return Err("Invalid data")
        }
        if bytes[*position] == b'}' {
            *position += 1;
            return Ok(Value::Object(map))
        }
        if bytes[*position] != b',' {
            return Err("Invalid token")
        }
        *position += 1;
        skip_whitespace(bytes, position);
        if *position == bytes.len() {
            return Err("Invalid data")
        }
    }
}

fn parse_string(bytes: &[u8], position: &mut usize) -> Result<Value, &'static str> {
    *position += 1;
    let start = *position;
    let mut skip = false;
    loop {
        if *position == bytes.len() {
            return Err("Invalid data")
        }
        if skip {
            *position += 1;
            skip = false;
            continue
        }
        match bytes[*position] {
            b'\\' => {
                *position += 1;
                skip = true;
                continue
            }
            b'"' => {
                *position += 1;
                return Ok(Value::String(String::from_utf8_lossy(&bytes[start..*position-1]).to_string()))
            }
            _ => {
                *position += 1;
            }
        }
    }

}

fn parse_array(bytes: &[u8], position: &mut usize) -> Result<Value, &'static str> {
    let mut array = Vec::new();
    *position += 1;
    skip_whitespace(bytes, position);
    if *position == bytes.len() {
        return Err("Invalid data")
    }
    if bytes[*position] == b']' {
        *position += 1;
        return Ok(Value::Array(array))
    }
    loop {
        array.push(parse_value(bytes, position)?);
        skip_whitespace(bytes, position);
        if *position == bytes.len() {
            return Err("Invalid data")
        }
        if bytes[*position] == b']' {
            *position += 1;
            return Ok(Value::Array(array))
        }
        if bytes[*position] != b',' {
            return Err("Invalid token")
        }
        *position += 1;
        skip_whitespace(bytes, position);
        if *position == bytes.len() {
            return Err("Invalid data")
        }
    }
}

fn parse_boolean(bytes: &[u8], position: &mut usize) -> Result<Value, &'static str> {
    match bytes[*position..*position + 4] {
        [b't', b'r', b'u', b'e'] => {
            *position += 4;
            return Ok(Value::Boolean(true))
        }
        [b'f', b'a', b'l', b's'] => {
            if bytes[*position + 4] != b'e' {
                return Err("Invalid token")
            }
            *position += 5;
            return Ok(Value::Boolean(false))
        }
        _ => {
            return Err("Invalid token")
    }
        
    }
}

fn parse_null(bytes: &[u8], position: &mut usize) -> Result<Value, &'static str> {
    if bytes[*position..*position + 4] == [b'n', b'u', b'l', b'l'] {
        *position += 4;
        return Ok(Value::Null)
    }
    Err("Invalid token")
}

fn parse_number(bytes: &[u8], position: &mut usize) -> Result<Value, &'static str> {
    let start = *position;
    let mut decimal = false;
    let mut exponent = false;
    if bytes[*position] == b'-' {
        *position += 1;
    }
    loop {
        match bytes[*position] {
            b'0'..=b'9' => {
                *position += 1;
            }
            b'.' => {
                if decimal {
                    return Err("Invalid token")
                }
                decimal = true;
                *position += 1;
            }
            b'e' | b'E' => {
                if exponent {
                    return Err("Invalid token")
                }
                exponent = true;
                *position += 1;
            }
            b'-' | b'+' => {
                if !exponent {
                    return Err("Invalid token")
                }
                *position += 1;
            }
            _ => break
        }
        if *position == bytes.len() {
            break;
        }
    }

    let value = String::from_utf8_lossy(&bytes[start..*position]).to_string();

    if decimal || exponent {
        Ok(Value::Float(match value.parse() {
            Ok(f) => f,
            Err(_) => return Err("Invalid token")
        }))
    } else {
        Ok(Value::Integer(match value.parse() {
            Ok(i) => i,
            Err(_) => return Err("Invalid token")
        }))
    }
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
            // Value::Binary(v) => format!("b'{}'", utils::encode(&v)),
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
