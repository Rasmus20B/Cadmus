
pub mod token;

use super::calculation::token::{Token, Function, GetArgument};
use std::str::Chars;

use crate::util::encoding_util::fm_string_decrypt;

use crate::emulator3::Emulator;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Calculation(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CalculationString(pub String);

impl Calculation {
    //fn decompile_calculation(bytecode: &[u8]) -> String {
    //    let mut it = bytecode.iter().peekable();
    //    let mut result = String::new();
    //    let mut in_get = false;
    //

    pub fn from_tokens(tokens: &Vec<Token>) -> Self {
        Calculation(tokens.iter()
            .map(|token| token.encode())
            .flatten()
            .collect())
    }

    pub fn from_text(code: &str) -> Self {
        Calculation::from_tokens(&lex_text(code))
    }

    fn lex(&self) -> Vec<Token> {
        let mut result = vec![];
        let mut ptr = 0;
        while ptr < self.0.len() {
            match self.0[ptr] {
                0x4 => {
                    result.push(Token::OpenParen)
                }
                0x5 => {
                    result.push(Token::CloseParen);
                }
                0x2d => {
                    result.push(Token::Function(Function::Abs))
                }
                0x9b => {
                    ptr += 1;
                    if self.0[ptr] == 0x9c {
                        ptr += 1;
                        match self.0[ptr] {
                            0x1d => {
                                result.push(Token::Function(Function::Get(GetArgument::CurrentTime)))
                            }
                            0x20 => {
                                result.push(Token::Function(Function::Get(GetArgument::AccountName)))
                            }
                            0x49 => {
                                result.push(Token::Function(Function::Get(GetArgument::DocumentsPath)))
                            }
                            0x5d => {
                                result.push(Token::Function(Function::Get(GetArgument::DocumentsPathListing)))
                            }
                            _ => {}
                        }
                    }
                }
                0x9d => {
                    result.push(Token::Function(token::Function::Acos));
                }
                0xfb => {
                    ptr += 1;
                    match self.0[ptr] {
                        0x3 => { result.push(Token::Function(Function::Char)) }
                        _ => eprintln!("unrecognized intrinsic.")
                    }
                }
                0x10 => {
                    /* decode number */
                    let end = ptr + 19;
                    while ptr < end {
                        let cur = self.0[ptr];
                        if self.0[ptr] == 8 {
                            result.push(Token::Number(cur as f64));
                            break;
                        }
                        ptr += 1;
                    }
                },
                0x13 => {
                    /* Processing String */
                    ptr += 1;
                    let len = self.0[ptr] as usize;
                    result.push(Token::String(String::from(&fm_string_decrypt(&self.0[ptr..ptr+len]))));
                }
                0x1a => {
                    /* decode variable */
                    ptr += 1;
                    let len = self.0[ptr] as usize;
                    result.push(Token::Variable(String::from(&fm_string_decrypt(&self.0[ptr..ptr+len]))));
                },
                0x25 => {
                    result.push(Token::Add);
                }
                0x26 => {
                    result.push(Token::Subtract);
                }
                0x27 => {
                    result.push(Token::Multiply);
                }
                0x28 => {
                    result.push(Token::Divide);
                },
                0x41 => {
                    result.push(Token::Less);
                }
                0x43 => {
                    result.push(Token::LessEqual);
                }
                0x44 => {
                    result.push(Token::Equal);
                }
                0x46 => {
                    result.push(Token::NotEqual);
                }
                0x47 => {
                    result.push(Token::GreaterEqual)
                }
                0x49 => {
                    result.push(Token::Greater);
                }
                0x50 => {
                    result.push(Token::Concatenate);
                }
                _ => {
                }
            }
        }
        result
    }

    pub fn eval(&self) -> String {
        todo!()
    }
}

pub fn lex_text(code: &str) -> Vec<Token> {
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
    use super::lex_text;

    use crate::dbobjects::schema::Schema;
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
        assert_eq!(lex_text(code), expected);

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
        assert_eq!(lex_text(code), expected);

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
        assert_eq!(lex_text(code), expected);

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
        assert_eq!(lex_text(code), expected);
    }
}





