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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_1() {
        let msg = "Test message".as_bytes();

        assert_eq!(encode(msg), "VGVzdCBtZXNzYWdl");
    }

    #[test]
    fn test_encode_2() {
        let msg = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".as_bytes();

        assert_eq!(encode(msg), "TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2NpbmcgZWxpdCwgc2VkIGRvIGVpdXNtb2QgdGVtcG9yIGluY2lkaWR1bnQgdXQgbGFib3JlIGV0IGRvbG9yZSBtYWduYSBhbGlxdWEuIFV0IGVuaW0gYWQgbWluaW0gdmVuaWFtLCBxdWlzIG5vc3RydWQgZXhlcmNpdGF0aW9uIHVsbGFtY28gbGFib3JpcyBuaXNpIHV0IGFsaXF1aXAgZXggZWEgY29tbW9kbyBjb25zZXF1YXQuIER1aXMgYXV0ZSBpcnVyZSBkb2xvciBpbiByZXByZWhlbmRlcml0IGluIHZvbHVwdGF0ZSB2ZWxpdCBlc3NlIGNpbGx1bSBkb2xvcmUgZXUgZnVnaWF0IG51bGxhIHBhcmlhdHVyLiBFeGNlcHRldXIgc2ludCBvY2NhZWNhdCBjdXBpZGF0YXQgbm9uIHByb2lkZW50LCBzdW50IGluIGN1bHBhIHF1aSBvZmZpY2lhIGRlc2VydW50IG1vbGxpdCBhbmltIGlkIGVzdCBsYWJvcnVtLg==");
    }

    #[test]
    fn test_encode_3() {
        let msg = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

        assert_eq!(encode(&msg), "AAECAwQFBgcICQoLDA0ODw==");
    }

    #[test]
    fn test_decode_1() {
        let msg = "VGVzdCBtZXNzYWdl";

        assert_eq!(decode(msg), "Test message".as_bytes());
    }

    #[test]
    fn test_decode_2() {
        let msg = "TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2NpbmcgZWxpdCwgc2VkIGRvIGVpdXNtb2QgdGVtcG9yIGluY2lkaWR1bnQgdXQgbGFib3JlIGV0IGRvbG9yZSBtYWduYSBhbGlxdWEuIFV0IGVuaW0gYWQgbWluaW0gdmVuaWFtLCBxdWlzIG5vc3RydWQgZXhlcmNpdGF0aW9uIHVsbGFtY28gbGFib3JpcyBuaXNpIHV0IGFsaXF1aXAgZXggZWEgY29tbW9kbyBjb25zZXF1YXQuIER1aXMgYXV0ZSBpcnVyZSBkb2xvciBpbiByZXByZWhlbmRlcml0IGluIHZvbHVwdGF0ZSB2ZWxpdCBlc3NlIGNpbGx1bSBkb2xvcmUgZXUgZnVnaWF0IG51bGxhIHBhcmlhdHVyLiBFeGNlcHRldXIgc2ludCBvY2NhZWNhdCBjdXBpZGF0YXQgbm9uIHByb2lkZW50LCBzdW50IGluIGN1bHBhIHF1aSBvZmZpY2lhIGRlc2VydW50IG1vbGxpdCBhbmltIGlkIGVzdCBsYWJvcnVtLg==";

        assert_eq!(decode(msg), "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".as_bytes());
    }

    #[test]
    fn test_decode_3() {
        let msg = "AAECAwQFBgcICQoLDA0ODw==";

        assert_eq!(decode(&msg), [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    }
}