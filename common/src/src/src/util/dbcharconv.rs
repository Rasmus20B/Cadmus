/* Double Byte Encoding/Decoding for scheme found in FileMaker File.
 * For now the encoding is a hardcoded lookup. */ 

pub const ENCODING_MAPPING: [(u8, u8, char); 29] = [
    (0x0, 0x0, '\0'), (0x2, 0xa, ' '), (0x2, 0x1d, '_'),
    (0x12, 0xf, 'a'), (0x12, 0x25, 'b'), (0x12, 0x3d, 'c'),
    (0x12, 0x50, 'd'), (0x12, 0x6b, 'e'), (0x12, 0xa3, 'f'),
    (0x12, 0xb0, 'g'), (0x12, 0xd3, 'h'), (0x12, 0xec, 'i'),
    (0x13, 0x5, 'j'), (0x13, 0x1e, 'k'), (0x13, 0x30, 'l'),
    (0x13, 0x5f, 'm'), (0x13, 0x6d, 'n'), (0x13, 0x8e, 'o'),
    (0x13, 0xb3, 'p'), (0x13, 0xc8, 'q'), (0x13, 0xda, 'r'),
    (0x14, 0x10, 's'), (0x14, 0x33, 't'), (0x14, 0x53, 'u'),
    (0x14, 0x7b, 'v'), (0x14, 0x8d, 'w'), (0x14, 0x97, 'x'),
    (0x14, 0x9c, 'y'), (0x14, 0xad, 'z')
];

pub fn decode_char(high: u8, low: u8) -> char {
    ENCODING_MAPPING.iter()
        .find(|&&(h, l, _)| h == high && l == low)
        .map(|&(_, _, ch) | ch)
        .unwrap_or('?')
}

pub fn decode_bytes(bytes: &[u8]) -> String {
    bytes.chunks_exact(2)
        .map(|b| decode_char(b[0], b[1]))
        .collect::<String>().strip_suffix("\0").unwrap().to_string()
}

pub fn encode_char(ch: char) -> (u8, u8) {
    ENCODING_MAPPING.iter()
        .find(|&&(_, _, c)| c == ch.to_ascii_lowercase())
        .map(|&(h, l, _) | (h, l))
        .unwrap_or((0, 0))
}

pub fn encode_text(text: &str) -> Vec<u8> {
    text.chars()
        .map(encode_char)
        .flat_map(|pair| [pair.0, pair.1].to_vec())
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::util::dbcharconv::{decode_char, encode_text};

    use super::encode_char;

    #[test]
    fn encode_test() {
        let text = "hello";
        let encoded : Vec<(u8, u8)> = text.chars()
            .map(encode_char)
            .collect();

        assert_eq!(
            vec![(0x12, 0xd3), (0x12, 0x6b), (0x13, 0x30), (0x13, 0x30), (0x13, 0x8e)],
            encoded
        );

        let decoded :String = encoded.iter()
            .map(|a| decode_char(a.0, a.1))
            .collect();


        assert_eq!(decoded, "hello");

        let text = "a_bcdefghijklmnopqrstuvwxy z\0\0";

        let encoded = [
            0x12, 0xf, 0x2, 0x1d, 0x12, 0x25, 0x12, 0x3d, 0x12, 0x50, 0x12, 0x6b,
            0x12, 0xa3, 0x12, 0xb0, 0x12, 0xd3, 0x12, 0xec, 0x13, 0x5, 0x13, 0x1e,
            0x13, 0x30, 0x13, 0x5f, 0x13, 0x6d, 0x13, 0x8e, 0x13, 0xb3, 0x13, 0xc8,
            0x13, 0xda, 0x14, 0x10, 0x14, 0x33, 0x14, 0x53,0x14, 0x7b, 0x14, 0x8d,
            0x14, 0x97, 0x14, 0x9c, 0x2, 0xa, 0x14, 0xad, 0x0, 0x0, 0x0, 0x0
        ];

        assert_eq!(
            encode_text(text),
            encoded
        );

        let out : String = [3, 1, 18, 15, 19, 48, 18, 236, 18, 176, 19, 109, 2, 10, 20, 51, 18, 107, 20,
        151, 20, 51, 3, 2, 0, 0, 0]
            .chunks(2)
            .map(|a| 
                if a[0] == 0 {
                    '\0'
                } else {
                    decode_char(a[0], a[1])
                })
            .collect();
        println!("{}", out);
    }
}



