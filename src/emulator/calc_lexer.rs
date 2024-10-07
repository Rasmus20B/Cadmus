
use core::fmt;

use super::calc_tokens::{self, Token, TokenType};

#[derive(Debug)]
pub enum LexerError {
    InvalidCharacter { character: char, position: usize },
    UnterminatedString { position: usize },
    UnexpectedEOF,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerError::InvalidCharacter { character, position } => {
                write!(f, "Invalid character '{}' @ position {}", character, position)
            },
            LexerError::UnterminatedString { position } => {
                write!(f, "Unterminated string starting @ position {}", position)
            },
            LexerError::UnexpectedEOF => {
                write!(f, "Unexpected end of file.")
            }
        }
    }
}

pub fn lex(calculation_string: &str) -> Result<Vec<calc_tokens::Token>, LexerError> {
    let flush_buffer = |b: &str| -> Result<calc_tokens::Token, LexerError> {
        match b {
            _ => {
                let n = b.parse::<f64>();
                if n.is_ok() {
                    Ok(calc_tokens::Token::with_value(calc_tokens::TokenType::NumericLiteral, n.unwrap().to_string()))
                } else if !b.as_bytes()[0].is_ascii_digit() {
                    Ok(calc_tokens::Token::with_value(calc_tokens::TokenType::Identifier, b.to_string()))
                } else {
                    Err(LexerError::InvalidCharacter { character: b.as_bytes()[0] as char, position: 0 })
                }
            }
        }
    };

    let mut tokens : Vec<calc_tokens::Token> = vec![];
    let mut lex_iter = calculation_string.chars().into_iter().peekable();
    let mut buffer = String::new();
    while let Some(c) = &lex_iter.next() {

        if c.is_whitespace() && buffer.is_empty() {
            continue;
        }

        match c {
            ' ' => {
                if !buffer.is_empty() {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
            },
            '(' => {
                if !buffer.is_empty() {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Token::new(calc_tokens::TokenType::OpenParen));
            },
            ')' => {
                if !buffer.is_empty() {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Token::new(calc_tokens::TokenType::CloseParen));
            },
            '+' => {
                if !buffer.is_empty() {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Token::new(calc_tokens::TokenType::Plus));
            }
            ',' => {
                if !buffer.is_empty() {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Token::new(calc_tokens::TokenType::Comma));
            },
            '-' => {
                if !buffer.is_empty() {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Token::new(calc_tokens::TokenType::Minus));
            },
            '*' => {
                if !buffer.is_empty() {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Token::new(calc_tokens::TokenType::Multiply));
            },
            '/' => {
                if !buffer.is_empty() {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Token::new(calc_tokens::TokenType::Divide));
            },
            '&' => {
                if !buffer.is_empty() {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Token::new(calc_tokens::TokenType::Ampersand));
            },
            '!' => {
                if !buffer.is_empty() {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                if *lex_iter.peek().unwrap() == '=' {
                    tokens.push(Token::new(calc_tokens::TokenType::Neq));
                    lex_iter.next();
                }
            },
            '=' => {
                if !buffer.is_empty() {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                if *lex_iter.peek().unwrap() == '=' {
                    tokens.push(Token::new(calc_tokens::TokenType::Eq));
                }
            },
            '<' => {
                if !buffer.is_empty() {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                if *lex_iter.peek().unwrap() == '=' {
                    tokens.push(Token::new(calc_tokens::TokenType::Ltq));
                } else {
                    tokens.push(Token::new(calc_tokens::TokenType::Lt));
                }
            },
            '>' => {
                if !buffer.is_empty() {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                if *lex_iter.peek().unwrap() == '=' {
                    tokens.push(Token::new(calc_tokens::TokenType::Gtq));
                } else {
                    tokens.push(Token::new(calc_tokens::TokenType::Gt));
                }
            }
            '"' => {
                // TODO: String handling directly from fmp12 file. Extra quotes.
                if !buffer.is_empty() {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                buffer.push(*c);
                let mut terminated = false;
                while let Some(c) = &lex_iter.next() {
                    if *c == '\"' {
                        terminated = true;
                        buffer.push(*c);
                        break;
                    }
                    buffer.push(*c);
                }

                if !terminated {
                    return Err(LexerError::UnterminatedString { position: 20 })
                }

                let size = buffer.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<Vec<_>>().len();
                if size > 0 {
                    tokens.push(Token::with_value(TokenType::String, buffer.clone()));
                }
                buffer.clear();
            },
            ':' => {
                buffer.push(*c);
                if buffer.is_empty() || *lex_iter.peek().unwrap_or(&'?') != ':' { 
                    return Err(LexerError::InvalidCharacter { character: ':', position: 20 })
                }
                lex_iter.next();
                buffer.push(*c);
                while let Some(c) = &lex_iter.next() {
                    buffer.push(*c);
                    let peeked = lex_iter.peek();
                    if peeked.is_none() || !peeked.unwrap().is_alphanumeric() {
                        break;
                    }
                }
                tokens.push(Token::with_value(TokenType::Identifier, buffer.clone()));
                buffer.clear();
            },
            _ => {
                buffer.push(*c);
            }
        }
    }
    if !buffer.is_empty() {
        let b = flush_buffer(buffer.as_str());
        buffer.clear();
        tokens.push(b.unwrap());
    }
    Ok(tokens)
}
