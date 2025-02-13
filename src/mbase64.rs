const CHARS: [char; 64] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '/'];
const PADDING: char = '=';

pub fn encode(bytes: &[u8]) -> String {

    let mut result = String::new();

    let mut iterator = bytes.chunks_exact(3);

    while let Some(chunk) = iterator.next() {
        result.push(CHARS[((chunk[0] & 0xFC) >> 2) as usize]);
        result.push(CHARS[(((chunk[0] & 0x03) << 4) | ((chunk[1] & 0xF0) >> 4)) as usize]);
        result.push(CHARS[(((chunk[1] & 0x0F) << 2) | ((chunk[2] & 0xC0) >> 6)) as usize]);
        result.push(CHARS[(chunk[2] & 0x3F) as usize]);
    }

    match iterator.remainder() {
        [a] => {
            result.push(CHARS[((a & 0xFC) >> 2) as usize]);
            result.push(CHARS[(((a & 0x03) << 4)) as usize]);
            result.push(PADDING);
            result.push(PADDING);
        },
        [a, b] => {
            result.push(CHARS[((a & 0xFC) >> 2) as usize]);
            result.push(CHARS[(((a & 0x03) << 4) | ((b & 0xF0) >> 4)) as usize]);
            result.push(CHARS[(((b & 0x0F) << 2)) as usize]);
            result.push(PADDING);
        },
        _ => {}
    }

    result
}

#[allow(unused)]
pub fn decode(bytes: &str) -> Vec<u8> {

    let mut result = Vec::new();

    let bytes = bytes.replace('=', "");
    let bytes = bytes.chars().collect::<Vec<char>>();

    let mut iterator = bytes.chunks_exact(4);

    while let Some(chunk) = iterator.next() {
        result.push(((CHARS.iter().position(|&c| c == chunk[0]).unwrap() as u8) << 2) | ((CHARS.iter().position(|&c| c == chunk[1]).unwrap() as u8) >> 4));
        result.push(((CHARS.iter().position(|&c| c == chunk[1]).unwrap() as u8) << 4) | ((CHARS.iter().position(|&c| c == chunk[2]).unwrap() as u8) >> 2));
        result.push(((CHARS.iter().position(|&c| c == chunk[2]).unwrap() as u8) << 6) | (CHARS.iter().position(|&c| c == chunk[3]).unwrap() as u8));
    }

    match iterator.remainder() {
        [a, b, d] => {
            result.push(((CHARS.iter().position(|&c| c == *a).unwrap() as u8) << 2) | ((CHARS.iter().position(|&c| c == *b).unwrap() as u8) >> 4));
            result.push(((CHARS.iter().position(|&c| c == *b).unwrap() as u8) << 4) | ((CHARS.iter().position(|&c| c == *d).unwrap() as u8) >> 2));
        },
        [a, b] => {
            result.push(((CHARS.iter().position(|&c| c == *a).unwrap() as u8) << 2) | ((CHARS.iter().position(|&c| c == *b).unwrap() as u8) >> 4));
        },
        [a] => {
            result.push(((CHARS.iter().position(|&c| c == *a).unwrap() as u8) << 2));
        },
        _ => {}
    }

    result
}