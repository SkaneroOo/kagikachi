

pub fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h0 = 0x67452301;
    let mut h1 = 0xEFCDAB89;
    let mut h2 = 0x98BADCFE;
    let mut h3 = 0x10325476;
    let mut h4 = 0xC3D2E1F0;
    let mut length = data.len() as u64;
    let prepared_length = (length + 9) + if (length + 9) % 64 == 0 { 0 } else { 64 - ((length + 9) % 64) };
    let mut prepared = vec![0; prepared_length as usize];

    for i in 0..data.len() {
        prepared[i] = data[i];
    }

    prepared[data.len()] = 0x80;
    length *= 8;
    prepared[(prepared_length as usize - 8)..].copy_from_slice(&length.to_be_bytes());

    for chunk in prepared.chunks(64) {
        let mut words = [0; 80];

        for i in 0..16 {
            words[i] = u32::from_be_bytes(chunk[i * 4..(i + 1) * 4].try_into().unwrap());
        }

        for i in 16..80 {
            words[i] = (words[i - 3] ^ words[i - 8] ^ words[i - 14] ^ words[i - 16]).rotate_left(1);
        }

        let mut a: u32 = h0;
        let mut b: u32 = h1;
        let mut c: u32 = h2;
        let mut d: u32 = h3;
        let mut e: u32 = h4;
        
        for i in 0..80 {
            let f;
            let k;
            match i {
                0..=19 => {
                    f = (b & c) | (!b & d);
                    k = 0x5A827999;
                }
                20..=39 => {
                    f = b ^ c ^ d;
                    k = 0x6ED9EBA1;
                }
                40..=59 => {
                    f = (b & c) | (b & d) | (c & d);
                    k = 0x8F1BBCDC;
                }
                60..=79 => {
                    f = b ^ c ^ d;
                    k = 0xCA62C1D6;
                }
                _ => unreachable!()
            }

            let temp = a.rotate_left(5)
                             .overflowing_add(f).0
                             .overflowing_add(e).0
                             .overflowing_add(k).0
                             .overflowing_add(words[i]).0;
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.overflowing_add(a).0;
        h1 = h1.overflowing_add(b).0;
        h2 = h2.overflowing_add(c).0;
        h3 = h3.overflowing_add(d).0;
        h4 = h4.overflowing_add(e).0;

    }

    let mut ret = [0; 20];
    ret.copy_from_slice([h0.to_be_bytes(), h1.to_be_bytes(), h2.to_be_bytes(), h3.to_be_bytes(), h4.to_be_bytes()].concat().as_slice());

    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_1() {
        let msg = "Test message".as_bytes();

        assert_eq!(sha1(msg), [141, 227, 155, 71, 34, 32, 127, 45, 162, 168, 49, 232, 115, 79, 2, 231, 64, 193, 87, 56]);
    }

    #[test]
    fn test_sha1_2() {
        let msg = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".as_bytes();

        assert_eq!(sha1(msg), [205, 54, 179, 112, 117, 138, 37, 155, 52, 132, 80, 132, 166, 204, 56, 71, 60, 185, 94, 39]);
    }

    #[test]
    fn test_sha1_3() {
        let msg = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

        assert_eq!(sha1(&msg), [86, 23, 139, 134, 165, 127, 172, 34, 137, 154, 153, 100, 24, 92, 44, 201, 110, 125, 165, 137]);
    }

    #[test]
    fn test_sha1_4() {
        let msg = [];

        assert_eq!(sha1(&msg), [218, 57, 163, 238, 94, 107, 75, 13, 50, 85, 191, 239, 149, 96, 24, 144, 175, 216, 7, 9]);
    }
}