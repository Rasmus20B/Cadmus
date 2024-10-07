
use super::calc_tokens;

pub fn lex(calculation_string: &str) -> Vec<calc_tokens::Token> {
    let flush_buffer = |b: &str| -> Result<calc_tokens::Token, String> {
        match b {
            _ => {
                let n = b.parse::<f64>();
                if n.is_ok() {
                    Ok(calc_tokens::Token::with_value(calc_tokens::TokenType::NumericLiteral, n.unwrap().to_string()))
                } else if !b.as_bytes()[0].is_ascii_digit() {
                    Ok(calc_tokens::Token::with_value(calc_tokens::TokenType::Identifier, b.to_string()))
                } else {
                    Err("Invalid Identifier".to_string())
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
                if buffer.len() > 0 {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
            },
            '(' => {
                if buffer.len() > 0 {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Ok::<calc_tokens::Token, String>(calc_tokens::Token::new(calc_tokens::TokenType::OpenParen)).unwrap());
            },
            ')' => {
                if buffer.len() > 0 {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Ok::<calc_tokens::Token, String>(calc_tokens::Token::new(calc_tokens::TokenType::CloseParen)).unwrap());
            },
            '+' => {
                if buffer.len() > 0 {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Ok::<calc_tokens::Token, String>(calc_tokens::Token::new(calc_tokens::TokenType::Plus)).unwrap());
            }
            ',' => {
                if buffer.len() > 0 {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Ok::<calc_tokens::Token, String>(calc_tokens::Token::new(calc_tokens::TokenType::Comma)).unwrap());
            },
            '-' => {
                if buffer.len() > 0 {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Ok::<calc_tokens::Token, String>(calc_tokens::Token::new(calc_tokens::TokenType::Minus)).unwrap());
            },
            '*' => {
                if buffer.len() > 0 {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Ok::<calc_tokens::Token, String>(calc_tokens::Token::new(calc_tokens::TokenType::Multiply)).unwrap());
            },
            '/' => {
                if buffer.len() > 0 {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Ok::<calc_tokens::Token, String>(calc_tokens::Token::new(calc_tokens::TokenType::Divide)).unwrap());
            },
            '&' => {
                if buffer.len() > 0 {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                tokens.push(Ok::<calc_tokens::Token, String>(calc_tokens::Token::new(calc_tokens::TokenType::Ampersand)).unwrap());
            },
            '!' => {
                if buffer.len() > 0 {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                if *lex_iter.peek().unwrap() == '=' {
                    tokens.push(Ok::<calc_tokens::Token, String>(
                            calc_tokens::Token::new(calc_tokens::TokenType::Neq)).unwrap());
                    lex_iter.next();
                }
            },
            '=' => {
                if buffer.len() > 0 {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                if *lex_iter.peek().unwrap() == '=' {
                    tokens.push(Ok::<calc_tokens::Token, String>(
                            calc_tokens::Token::new(calc_tokens::TokenType::Eq)).unwrap());
                }
            },
            '<' => {
                if buffer.len() > 0 {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                if *lex_iter.peek().unwrap() == '=' {
                    tokens.push(Ok::<calc_tokens::Token, String>(
                            calc_tokens::Token::new(calc_tokens::TokenType::Ltq)).unwrap());
                } else {
                    tokens.push(Ok::<calc_tokens::Token, String>(
                            calc_tokens::Token::new(calc_tokens::TokenType::Lt)).unwrap());
                }
            },
            '>' => {
                if buffer.len() > 0 {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                if *lex_iter.peek().unwrap() == '=' {
                    tokens.push(Ok::<calc_tokens::Token, String>(
                            calc_tokens::Token::new(calc_tokens::TokenType::Gtq)).unwrap());
                } else {
                    tokens.push(Ok::<calc_tokens::Token, String>(
                            calc_tokens::Token::new(calc_tokens::TokenType::Gt)).unwrap());
                }
            }
            '"' => {
                // TODO: String handling directly from fmp12 file. Extra quotes.
                if buffer.len() > 0 {
                    let b = flush_buffer(buffer.as_str());
                    buffer.clear();
                    tokens.push(b.unwrap());
                }
                buffer.push(*c);
                while let Some(c) = &lex_iter.next() {
                    if *c == '\"' {
                        buffer.push(*c);
                        break;
                    }
                    buffer.push(*c);
                }

                let size = buffer.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<Vec<_>>().len();
                if size > 0 {
                    tokens.push(calc_tokens::Token::with_value(calc_tokens::TokenType::String, buffer.clone()));
                }
                buffer.clear();
            },
            ':' => {
                buffer.push(*c);
                if buffer.is_empty() || *lex_iter.peek().unwrap_or(&'?') != ':' { 
                    eprintln!("invalid ':' found.");
                    buffer.push(*c);
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
                tokens.push(calc_tokens::Token::with_value(calc_tokens::TokenType::Identifier, buffer.clone()));
                buffer.clear();
            },
            _ => {
                buffer.push(*c);
            }
        }
    }

    if buffer.len() > 0 {
        let b = flush_buffer(buffer.as_str());
        buffer.clear();
        tokens.push(b.unwrap());
    }

    tokens
}
