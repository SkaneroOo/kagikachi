

pub fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h0 = 0x67452301;
    let mut h1 = 0xEFCDAB89;
    let mut h2 = 0x98BADCFE;
    let mut h3 = 0x10325476;
    let mut h4 = 0xC3D2E1F0;
    let mut length = data.len() as u64;
    let prepared_length = (length + 9) + if (length + 9) % 64 == 0 { 0 } else { 64 - ((length + 9) % 64) };
    println!("Prepared length: {}", prepared_length);
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