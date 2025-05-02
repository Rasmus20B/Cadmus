

pub fn get_path_int(bytes : &[u8]) -> usize {
    match bytes.len() {
        1 => bytes[0] as usize,
        2 => 0x80 + ((bytes[0] as usize) & 0x7f << 8) + bytes[1] as usize,
        3 => 0x80 + ((bytes[0] as usize) & 0x7f << 8) + ((bytes[1] as usize) & 0x7f),
        _ => 0,
    }
}

pub fn put_path_int(n: u32) -> Vec<u8> {
    if n >= 128 {
        let x = n - 128;
        let mut b1 = (((x >> 8) & 0x7f) | 0x80) as u8;
        let b2 = (x & 0xFF) as u8;
        b1 |= 0x7c;
        [b1, b2].to_vec()
    } else {
        [n as u8].to_vec() 
    }
}

pub fn get_int(bytes: &[u8]) -> usize {
    match bytes.len() {
        1 => bytes[0] as usize,
        2 => ((bytes[0] as usize) << 8) + (bytes[1] as usize),
        4 => (get_int(&bytes[0..2]) << 16) + get_int(&bytes[2..4]),
        _ => 0
    }
}

pub fn put_int(mut n: usize) -> Vec<u8> {
    let mut res = vec![0, 0, 0, 0];
    let mut idx = 3;
    while n > 0 {
        let cur = n % 256;
        n /= 256;
        res[idx] = cur as u8;
        idx -= 1;
    }
    res
}

pub fn fm_string_encrypt(text: &str) -> Vec<u8> {
    text
        .bytes()
        .map(|c| c ^ 0x5A)
        .collect()
}

pub fn fm_string_decrypt(bytes: &[u8]) -> String {
    match String::from_utf8(bytes
                                 .iter()
                                 .map(|c| c ^ 0x5A)
                                 .collect::<Vec<u8>>()) {
        Ok(v) => v.to_string(),
        Err(_) => "value not utf-8.".to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::encoding_util::*;

    #[test]
    fn path_int_testing() {
        assert_eq!(get_path_int(&[1]), 1);
        assert_eq!(get_path_int(&[127]), 127);
        assert_eq!(get_path_int(&[128]), 128);
        assert_eq!(get_path_int(&[252, 1]), 129);
        let n = put_path_int(129);
        println!("{:#010b} == {:#010b} :: {:#010b} == {:#010b}", n[0], 252, n[1], 1);
        assert_eq!(&put_path_int(129), &[252 , 1]);
        assert_eq!(&put_path_int(130), &[252 , 2]);
        assert_eq!(&put_path_int(131), &[252 , 3]);
        assert_eq!(&put_path_int(1), &[1]);
        assert_eq!(&put_path_int(2), &[2]);
        assert_eq!(get_path_int(&[]), 0);
        assert_eq!(get_path_int(&[252, 138]), 266);
        assert_eq!(get_path_int(&put_path_int(266)), 266);
        assert_eq!(get_path_int(&[128, 138]), 266);
        assert_eq!(get_path_int(&[252, 1]), 129);
        assert_eq!(&put_path_int(0), &[0]);
    }

    #[test]
    fn int_testing() {
        assert_eq!(put_int(67), &[0, 0, 0, 67]);
        assert_eq!(get_int(&[0, 0, 0, 67]), 67);
        assert_eq!(get_int(&[0, 0, 4, 0]), 1024);
        assert_eq!(put_int(1024), &[0, 0, 4, 0]);
    }

    #[test]
    fn string_testing() {
        assert_eq!(fm_string_decrypt(&[0x7e, 0x22]), "$x");
        assert_eq!(fm_string_decrypt(&[0x7e, 0x23]), "$y");
        assert_eq!(fm_string_decrypt(&[0x19, 0x3b, 0x34, 0x39, 0x3f, 0x36]), "Cancel");
        assert_eq!(fm_string_decrypt(&[0x32, 0x3f, 0x36, 0x36, 0x35]), "hello");
        assert_eq!(fm_string_decrypt(&[]), "");
    }
}

