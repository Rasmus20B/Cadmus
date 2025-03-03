use super::encoding_util::fm_string_decrypt;

pub fn decompile_calculation(bytecode: &[u8]) -> String {
    let mut it = bytecode.iter().peekable();
    let mut result = String::new();

    while let Some(c) = it.next() {
        match c {
            0x4 => {
                result.push('(');
            }
            0x5 => {
                result.push(')');
            }
            0x2d => {
                result.push_str("Abs");
            }
            0x9b => {
                result.push_str("Get");
            }
            0x9c => {
                match it.next().unwrap() {
                    0x1d => {
                        result.push_str("CurrentTime");
                    }
                    0x20 => {
                        result.push_str("AccountName");
                    }
                    0x49 => {
                        result.push_str("DocumentsPath");
                    }
                    0x5d => {
                        result.push_str("DocumentsPathListing");
                    }
                    _ => {}
                }
            }
            0x9d => {
                result.push_str("Acos");
            }
            0xfb => {
                match it.next().unwrap() {
                    0x3 => { result.push_str("Char")}
                    _ => eprintln!("unrecognized intrinsic.")
                }
            }
            0x10 => {
                /* decode number */
                for i in 0..19 {
                    let cur = it.next();
                    if i == 8 {
                        result.push_str(&cur.unwrap().to_string());
                    }
                }
            },
            0x13 => {
                /* Processing String */
                let n = it.next();
                let mut s = String::new();
                for _ in 1..=*n.unwrap() as usize {
                    s.push(*it.next().unwrap() as char);
                }
                let mut text = String::new();
                text.push('"');
                text.push_str(&fm_string_decrypt(s.as_bytes()));
                text.push('"');

                result.push_str(&text);
            }
            0x1a => {
                /* decode variable */
                let n = it.next();
                let mut name_arr = String::new();
                for _ in 1..=*n.unwrap() as usize {
                    name_arr.push(*it.next().unwrap() as char);
                }
                let name = fm_string_decrypt(name_arr.as_bytes());
                result.push_str(&name);
            },
            0x25 => {
                result.push('+');
            }
            0x26 => {
                result.push('-');
            }
            0x27 => {
                result.push('*');
            }
            0x28 => {
                result.push('/');
            },
            0x41 => {
                result.push('<');
            }
            0x43 => {
                result.push_str("<=");
            }
            0x44 => {
                result.push_str("==");
            }
            0x46 => {
                result.push_str("!=");
            }
            0x47 => {
                result.push_str(">=");
            }
            0x49 => {
                result.push('>');
            }
            0x50 => {
                result.push('&');
            }
            0xC => {
                result.push(' ');
            }
            _ => {

            }
        }

    }
    return result;
}

