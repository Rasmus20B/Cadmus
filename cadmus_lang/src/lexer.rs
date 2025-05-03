use std::iter::Peekable;

use crate::{
    error::Result,
    token::{match_keyword, SourceLoc, Token, TokenValue, KEYWORD_MAP},
};

struct LexIter<'a> {
    input: &'a str,
    line: u32,
    column: u32,
    chars: Peekable<std::str::Chars<'a>>,
}

impl<'a> LexIter<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            line: 1,
            column: 0,
            chars: input.chars().peekable(),
        }
    }
    fn peek(&mut self) -> Option<(char, u32, u32)> {
        if let Some(c) = self.chars.peek() {
            let mut line = self.line;
            let mut column = self.column;
            if *c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
            Some((*c, line, column))
        } else {
            None
        }
    }
}

impl<'a> Iterator for LexIter<'a> {
    type Item = (char, u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.chars.next() {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some((c, self.line, self.column))
        } else {
            None
        }
    }
}

pub fn lex(text: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::<Token>::new();
    let mut iter = LexIter::new(text);
    while let Some((c, line, col)) = iter.next() {
        match c {
            c if c.is_alphabetic() => {
                let mut buffer = String::new();
                buffer.push(c);

                while let Some(&next_c) = iter.chars.peek() {
                    if next_c.is_alphanumeric() {
                        if let Some((c, _, _)) = iter.next() {
                            buffer.push(c);
                        }
                    } else if next_c.is_whitespace() {
                        if let Some(keyword) = match_keyword(&buffer) {
                            tokens.push(Token::new(keyword, SourceLoc::new(line, col)));
                            iter.next(); // consume the whitespace
                            buffer.clear();
                            break;
                        } else {
                            buffer.push(next_c);
                            iter.next(); // include space in identifier
                        }
                    } else {
                        break;
                    }
                }

                if !buffer.is_empty() {
                    if let Some(keyword) = match_keyword(&buffer) {
                        tokens.push(Token::new(keyword, SourceLoc::new(line, col)));
                    } else {
                        tokens.push(Token::new(
                            TokenValue::Identifier(buffer.trim().to_string()),
                            SourceLoc::new(line, col),
                        ));
                    }
                }
            }
            '%' => {
                let mut n_buffer = String::new();
                while let Some(&next_c) = iter.chars.peek() {
                    if next_c.is_alphanumeric() {
                        if let Some((c, _, _)) = iter.next() {
                            n_buffer.push(c);
                        }
                    } else {
                        break;
                    }
                }

                let id = n_buffer.parse::<u32>().unwrap();
                tokens.push(Token::new(
                    TokenValue::ObjectNumber(id),
                    SourceLoc::new(line, col),
                ));
            }
            '=' => tokens.push(Token::new(
                TokenValue::Assignment,
                SourceLoc::new(line, col),
            )),
            '{' => tokens.push(Token::new(TokenValue::OpenBrace, SourceLoc::new(line, col))),
            '}' => tokens.push(Token::new(
                TokenValue::CloseBrace,
                SourceLoc::new(line, col),
            )),
            _ => {}
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use crate::token::TokenValue;

    use super::lex;

    #[test]
    fn multi_word_identifier() {
        let text = "table %1 Quotes Machines 2";
        let tokens = lex(text).unwrap();
        assert_eq!(tokens[0].token_val, TokenValue::Table);
        assert_eq!(tokens[1].token_val, TokenValue::ObjectNumber(1));
        assert_eq!(
            tokens[2].token_val,
            TokenValue::Identifier(String::from("Quotes Machines 2"))
        );
    }

    #[test]
    fn multi_word_identifier_after_keyword() {
        let text = "table Quotes Machines %1 Quotes Machines 2";
        let tokens = lex(text).unwrap();
        assert_eq!(tokens[0].token_val, TokenValue::Table);
        assert_eq!(
            tokens[1].token_val,
            TokenValue::Identifier(String::from("Quotes Machines"))
        );
        assert_eq!(tokens[2].token_val, TokenValue::ObjectNumber(1));
        assert_eq!(
            tokens[3].token_val,
            TokenValue::Identifier(String::from("Quotes Machines 2"))
        );
    }

    #[test]
    fn keyword_repeated() {
        let text = "table table table %1 Quotes = {}";
        let tokens = lex(text).unwrap();
        assert_eq!(tokens[0].token_val, TokenValue::Table);
        assert_eq!(tokens[1].token_val, TokenValue::Table);
        assert_eq!(tokens[2].token_val, TokenValue::Table);
        assert_eq!(tokens[3].token_val, TokenValue::ObjectNumber(1));
        assert_eq!(
            tokens[4].token_val,
            TokenValue::Identifier(String::from("Quotes"))
        );
        assert_eq!(tokens[5].token_val, TokenValue::Assignment);
        assert_eq!(tokens[6].token_val, TokenValue::OpenBrace);
        assert_eq!(tokens[7].token_val, TokenValue::CloseBrace);
    }
}
