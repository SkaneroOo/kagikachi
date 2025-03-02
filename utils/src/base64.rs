const CHARS: [char; 64] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '/'];
const REV_CHARS: [u8; 80] = [62, 0, 0, 0, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 0, 0, 0, 0, 0, 0, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51];
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

    let bytes = bytes.chars().filter(|&c| c != PADDING).map(|c| (c as u8 - b'+') as usize).collect::<Vec<usize>>();

    let mut iterator = bytes.chunks_exact(4);

    while let Some(chunk) = iterator.next() {
        result.push((REV_CHARS[chunk[0]] << 2) | ((REV_CHARS[chunk[1]] >> 4)));
        result.push((REV_CHARS[chunk[1]] << 4) | ((REV_CHARS[chunk[2]] >> 2)));
        result.push((REV_CHARS[chunk[2]] << 6) | ( REV_CHARS[chunk[3]]      ));
    }

    match iterator.remainder() {
        [a, b, c] => {
            result.push((REV_CHARS[*a] << 2) | ((REV_CHARS[*b] >> 4)));
            result.push((REV_CHARS[*b] << 4) | ((REV_CHARS[*c] >> 2)));
        },
        [a, b] => {
            result.push((REV_CHARS[*a ] << 2) | ((REV_CHARS[*b] >> 4)));
        },
        [a] => {
            result.push(REV_CHARS[*a] << 2);
        },
        _ => {}
    }

    result
}