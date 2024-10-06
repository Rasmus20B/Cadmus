
use crate::burn_script::tokens::*;

#[derive(Debug, PartialEq)]
pub struct Lexer {
    code: String,
}

impl Lexer {
    pub fn new(text: String) -> Self {
        Self { code: text }
    }

    pub fn get_tokens(&self) -> Vec<Token> {
        let mut result = vec![];
        let mut buffer = String::new();
        let mut lex_iter = self.code.chars().into_iter().peekable();
        let mut scope = 0;

        let flush_buffer = |b: &str| {
            match b {
                "define" => Token::new(TokenType::Define),
                "let" => Token::new(TokenType::Let),
                "loop" => Token::new(TokenType::Loop),
                "if" => Token::new(TokenType::If),
                "elif" => Token::new(TokenType::Elif),
                "else" => Token::new(TokenType::Else),
                _ => {
                    let n = b.parse::<f64>();
                    if n.is_ok() {
                        Token::with_value(TokenType::NumericLiteral, b)
                    } else {
                        Token::with_value(TokenType::Identifier, b)
                    }
                }
            }
        };
        while let Some(c) = &lex_iter.next() {
            if buffer.is_empty() && c.is_whitespace() {
                continue;
            }
            let b = buffer.as_str();

            let tmp: Option<Vec<_>> = match c {
                x if x.is_whitespace() =>  
                {
                    let mut ret: Vec<Token> = vec![];
                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }
                    Some(ret)
                },
                '(' =>
                {
                    let mut ret: Vec<Token> = vec![];

                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }

                    let mut stack = 0;
                    let mut in_string = false;
                    while let Some(c) = &lex_iter.next() {
                        match c {
                            ' ' => {
                                if !in_string && buffer.is_empty() {
                                    continue;
                                } else {
                                    buffer.push(*c);
                                }
                            }
                            '\"' => {
                                in_string = !in_string;
                                buffer.push(*c);
                            }
                            ')' => {
                                if stack == 0 {
                                    ret.push(Token::with_value(TokenType::Argument, &buffer));
                                    buffer.clear();
                                    break;
                                } else {
                                    buffer.push(*c);
                                    stack -= 1;
                                }
                            },
                            '(' => {
                                stack += 1;
                                buffer.push(*c);
                            }
                            ',' => {
                                if stack == 0 {
                                    ret.push(Token::with_value(TokenType::Argument, &buffer));
                                    buffer.clear();
                                } else {
                                    buffer.push(*c);
                                }
                            }
                            _ => {
                                buffer.push(*c);
                            }
                        }
                    }
                    Some(ret)
                },
                '{' =>
                {
                    let mut ret: Vec<Token> = vec![];
                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }
                    ret.push(Token::new(TokenType::OpenBracket));
                    scope += 1;
                    Some(ret)
                },
                '}' =>
                {
                    let mut ret: Vec<Token> = vec![];
                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }
                    ret.push(Token::new(TokenType::CloseBracket));
                    scope -= 1;
                    Some(ret)
                },
                ',' => 
                {

                    let mut ret: Vec<Token> = vec![];

                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }
                    ret.push(Token::new(TokenType::Comma));
                    Some(ret)
                },
                ';' =>
                {
                    let mut ret: Vec<Token> = vec![];
                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }
                    ret.push(Token::new(TokenType::SemiColon));
                    Some(ret)
                }
                '!' =>  
                {
                    let mut ret: Vec<Token> = vec![];
                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }
                    if *lex_iter.peek().unwrap() != '=' {
                        ret.push(Token::new(TokenType::Assign));
                    } else {
                        lex_iter.next();
                        ret.push(Token::new(TokenType::Neq));
                    }
                    Some(ret)
                }
                '=' =>  
                {
                    let mut ret: Vec<Token> = vec![];
                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }
                    if *lex_iter.peek().unwrap() != '=' {
                        ret.push(Token::new(TokenType::Assign));
                    } else {
                        lex_iter.next();
                        ret.push(Token::new(TokenType::Eq));
                    }
                    Some(ret)
                },
                '>' =>  
                {
                    let mut ret: Vec<Token> = vec![];
                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }
                    if *lex_iter.peek().unwrap() == '=' {
                        lex_iter.next();
                        ret.push(Token::new(TokenType::Geq));
                    } else {
                        lex_iter.next();
                        ret.push(Token::new(TokenType::Gt));
                    }
                    Some(ret)
                }
                '<' =>  
                {
                    let mut ret: Vec<Token> = vec![];
                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }
                    if *lex_iter.peek().unwrap() == '=' {
                        lex_iter.next();
                        ret.push(Token::new(TokenType::Leq));
                    } else {
                        lex_iter.next();
                        ret.push(Token::new(TokenType::Lt));
                    }
                    Some(ret)
                }
                '+' =>
                {
                    let mut ret: Vec<Token> = vec![];
                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }
                    ret.push(Token::new(TokenType::Plus));
                    Some(ret)
                },
                '-' =>
                {
                    let mut ret: Vec<Token> = vec![];
                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }
                    ret.push(Token::new(TokenType::Minus));
                    Some(ret)
                },
                '*' =>
                {
                    let mut ret: Vec<Token> = vec![];
                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }
                    ret.push(Token::new(TokenType::Multiply));
                    Some(ret)
                },
                '/' =>
                {
                    let mut ret: Vec<Token> = vec![];
                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }
                    ret.push(Token::new(TokenType::Divide));
                    Some(ret)
                },
                '&' =>
                {
                    let mut ret: Vec<Token> = vec![];
                    if buffer.len() > 0 {
                        let b = flush_buffer(buffer.as_str());
                        buffer.clear();
                        ret.push(b);
                    }
                    ret.push(Token::new(TokenType::Ampersand));
                    Some(ret)
                },
                _ => {
                    buffer.push(*c);
                    None
                }
            };

            if tmp.is_some() {
                for t in tmp.unwrap() {
                    result.push(t);
                }
                buffer.clear();
            }
        }

        return result;
    }
}

#[cfg(test)]
mod tests {
    use crate::burn_script::tokens::*;
    use super::Lexer;
    #[test]
    fn lex_test_basic() {
        let code = "
        define test_func(x, y) {
            set_variable(i, x);
            loop {
                exit_loop_if(i == y);
                set_variable(i, i + 1);
            }
            set_variable(x, min(3, 3));
            assert(i == y);
            exit_script(i);
        }";

        let tokens_expected: Vec<Token> = vec![
            Token::new(TokenType::Define),
            Token::with_value(TokenType::Identifier, "test_func"),
            Token::with_value(TokenType::Argument, "x"),
            Token::with_value(TokenType::Argument, "y"),
            Token::new(TokenType::OpenBracket),
            Token::with_value(TokenType::Identifier, "set_variable"),
            Token::with_value(TokenType::Argument, "i"),
            Token::with_value(TokenType::Argument, "x"),
            Token::new(TokenType::SemiColon),
            Token::new(TokenType::Loop),
            Token::new(TokenType::OpenBracket),
            Token::with_value(TokenType::Identifier, "exit_loop_if"),
            Token::with_value(TokenType::Argument, "i == y"),
            Token::new(TokenType::SemiColon),
            Token::with_value(TokenType::Identifier, "set_variable"),
            Token::with_value(TokenType::Argument, "i"),
            Token::with_value(TokenType::Argument, "i + 1"),
            Token::new(TokenType::SemiColon),
            Token::new(TokenType::CloseBracket),
            Token::with_value(TokenType::Identifier, "set_variable"),
            Token::with_value(TokenType::Argument, "x"),
            Token::with_value(TokenType::Argument, "min(3, 3)"),
            Token::new(TokenType::SemiColon),
            Token::with_value(TokenType::Identifier, "assert"),
            Token::with_value(TokenType::Argument, "i == y"),
            Token::new(TokenType::SemiColon),
            Token::with_value(TokenType::Identifier, "exit_script"),
            Token::with_value(TokenType::Argument, "i"),
            Token::new(TokenType::SemiColon),
            Token::new(TokenType::CloseBracket)
        ];
        let tokens_actual = Lexer::new(code.to_string()).get_tokens();

        /* print non-matching tokens */
        for (i, t) in tokens_expected.iter()
            .enumerate()
                .zip(&tokens_actual)
                .filter(|x| *x.0.1 != *x.1) {
            println!("{}. expected: {:?}    :::     actual: {:?}", i.0, i.1, t);
        }
        assert_eq!(tokens_expected, tokens_actual);
    }
}

