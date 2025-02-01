
pub mod token;

use super::calculation::token::Token;
use std::str::Chars;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Calculation(Vec<u8>);
pub struct CalcString(Vec<u8>);

impl Calculation {
    //fn decompile_calculation(bytecode: &[u8]) -> String {
    //    let mut it = bytecode.iter().peekable();
    //    let mut result = String::new();
    //    let mut in_get = false;
    //
    //    while let Some(c) = it.next() {
    //        match c {
    //            0x4 => {
    //                result.push('(');
    //            }
    //            0x5 => {
    //                result.push(')');
    //            }
    //            0x2d => {
    //                result.push_str("Abs");
    //            }
    //            0x9b => {
    //                result.push_str("Get");
    //            }
    //            0x9c => {
    //                match it.next().unwrap() {
    //                    0x1d => {
    //                        result.push_str("CurrentTime");
    //                    }
    //                    0x20 => {
    //                        result.push_str("AccountName");
    //                    }
    //                    0x49 => {
    //                        result.push_str("DocumentsPath");
    //                    }
    //                    0x5d => {
    //                        result.push_str("DocumentsPathListing");
    //                    }
    //                    _ => {}
    //                }
    //            }
    //            0x9d => {
    //                result.push_str("Acos");
    //            }
    //            0xfb => {
    //                match it.next().unwrap() {
    //                    0x3 => { result.push_str("Char")}
    //                    _ => eprintln!("unrecognized intrinsic.")
    //                }
    //            }
    //            0x10 => {
    //                /* decode number */
    //                for i in 0..19 {
    //                    let cur = it.next();
    //                    if i == 8 {
    //                        result.push_str(&cur.unwrap().to_string());
    //                    }
    //                }
    //            },
    //            0x13 => {
    //                /* Processing String */
    //                let n = it.next();
    //                let mut s = String::new();
    //                for i in 1..=*n.unwrap() as usize {
    //                    s.push(*it.next().unwrap() as char);
    //                }
    //                let mut text = String::new();
    //                text.push('"');
    //                text.push_str(&fm_string_decrypt(s.as_bytes()));
    //                text.push('"');
    //
    //                result.push_str(&text);
    //            }
    //            0x1a => {
    //                /* decode variable */
    //                let n = it.next();
    //                let mut name_arr = String::new();
    //                for i in 1..=*n.unwrap() as usize {
    //                    name_arr.push(*it.next().unwrap() as char);
    //                }
    //                let name = fm_string_decrypt(name_arr.as_bytes());
    //                result.push_str(&name);
    //            },
    //            0x25 => {
    //                result.push('+');
    //            }
    //            0x26 => {
    //                result.push('-');
    //            }
    //            0x27 => {
    //                result.push('*');
    //            }
    //            0x28 => {
    //                result.push('/');
    //            },
    //            0x41 => {
    //                result.push('<');
    //            }
    //            0x43 => {
    //                result.push_str("<=");
    //            }
    //            0x44 => {
    //                result.push_str("==");
    //            }
    //            0x46 => {
    //                result.push_str("!=");
    //            }
    //            0x47 => {
    //                result.push_str(">=");
    //            }
    //            0x49 => {
    //                result.push('>');
    //            }
    //            0x50 => {
    //                result.push('&');
    //            }
    //            0xC => {
    //                result.push(' ');
    //            }
    //            _ => {
    //
    //            }
    //        }
    //
    //    }
    //    return result;
    //}


    pub fn from_text(code: &str) -> Self {
        let tokens = lex(code);

        Calculation(tokens.iter()
            .map(|token| token.encode())
            .flatten()
            .collect())
    }

    pub fn eval() -> String {
        todo!()
    }
}

pub fn lex(code: &str) -> Vec<Token> {
    let mut result = vec![];
    let mut iter = code.chars().into_iter().peekable();
    let mut buffer = String::new();

    while let Some(ch) = iter.next() {
        match ch {
            '$' => {
                // Clear buffer first
                buffer.push(ch);
                while let Some(c) = iter.peek() {
                    match c {
                        c if c.is_alphanumeric() => {
                            buffer.push(*c);
                        }
                        _ => {
                            break;
                        }
                    }
                    iter.next();
                }

                if !buffer.is_empty() {
                    result.push(Token::Variable(buffer.clone()));
                } 
            }
            '"' => {
                while let Some(c) = iter.next() {
                    if c == '"' {
                        break;
                    } else {
                        buffer.push(c)
                    }
                }
                result.push(Token::String(buffer.clone()));
                buffer.clear();
            }
            ';' => {
                result.push(Token::SemiColon)
            }
            '=' => {
                if *iter.peek().unwrap() == '=' {
                    result.push(Token::Equal)
                }
                iter.next();
            }
            '(' => {
                result.push(Token::OpenParen);
            }
            ')' => {
                result.push(Token::CloseParen);
            }
            '+' => {
                result.push(Token::Add);
            }
            '-' => {
                result.push(Token::Subtract);
            }
            '*' => {
                result.push(Token::Multiply);
            }
            '/' => {
                result.push(Token::Divide);
            }
            '&' => {
                result.push(Token::Concatenate);
            }
            '!' => {
                if *iter.peek().unwrap() == '=' {
                    result.push(Token::NotEqual);
                    iter.next();
                } else {
                    result.push(Token::Negate)
                }
            }
            c if c.is_numeric() => {
                buffer.push(c);
                while let Some(name_char) = iter.peek() {
                    println!("char: {}", name_char);
                    if name_char.is_numeric() {
                        buffer.push(*name_char);
                        iter.next();
                    } else {
                        break;
                    }
                }
                println!("buffer: {}", buffer);
                result.push(Token::Number(buffer.parse::<f64>().unwrap().clone()))
            }
            c if c.is_alphabetic() => {
                buffer.push(c);
                let mut tmp = String::new();
                while let Some(c) = iter.peek() {
                    match c {
                        c if c.is_alphanumeric() => {
                            buffer.push(*c);
                        }
                        ':' => {
                            iter.next();
                            if *iter.peek().unwrap() == ':' {
                                tmp.extend(buffer.chars());
                                buffer.clear();
                            } else {
                                // ERROR
                            }
                        }
                        _ => {
                            break;
                        }
                    }
                    iter.next();
                }

                if !tmp.is_empty() {
                    result.push(Token::FieldReference(tmp, buffer.clone()));
                } else {
                    result.push(Token::Identifier(buffer.clone()));
                }
            }
            ' ' => {
                result.push(Token::Space)
            }
            _ => {

            }
        }
        buffer.clear();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::token::Token;
    use super::Calculation;
    use super::lex;
    #[test]
    fn calc_encoding_test() {
        let calculation = "$x + 10";
        let expected = Calculation(vec![26, 2, 126, 34, // $x
            12, 19, 1, 122, 0, // space
            37, // +
            12, 19, 1, 122, 0, // space
            16, 2, 0, 1, 0, 16, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32 // 10
        ]); 
        let calc = Calculation::from_text(calculation);
        assert_eq!(calc, expected);
    }
    #[test]
    fn lex_test() {
        let code = "$x == 10";
        let expected = vec![
            Token::Variable(String::from("$x")),
            Token::Space,
            Token::Equal,
            Token::Space,
            Token::Number(10.0)
        ];
        assert_eq!(lex(code), expected);

        let code = "!($x != 10)";
        let expected = vec![
            Token::Negate,
            Token::OpenParen,
            Token::Variable(String::from("$x")),
            Token::Space,
            Token::NotEqual,
            Token::Space,
            Token::Number(10.0),
            Token::CloseParen,
        ];
        assert_eq!(lex(code), expected);

        let code = "!(Person::FirstName != \"Kevin\")";
        let expected = vec![
            Token::Negate,
            Token::OpenParen,
            Token::FieldReference(String::from("Person"), String::from("FirstName")),
            Token::Space,
            Token::NotEqual,
            Token::Space,
            Token::String(String::from("Kevin")),
            Token::CloseParen,
        ];
        assert_eq!(lex(code), expected);

        let code = "!(GetRepetition(Person::FirstName; 4) != \"Kevin\")";
        let expected = vec![
            Token::Negate,
            Token::OpenParen,
            Token::Identifier(String::from("GetRepetition")),
            Token::OpenParen,
            Token::FieldReference(String::from("Person"), String::from("FirstName")),
            Token::SemiColon,
            Token::Space,
            Token::Number(4.0),
            Token::CloseParen,
            Token::Space,
            Token::NotEqual,
            Token::Space,
            Token::String(String::from("Kevin")),
            Token::CloseParen,
        ];
        assert_eq!(lex(code), expected);
    }
}





