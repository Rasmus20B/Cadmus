
use crate::compile::token::*;

pub fn tokenize(code: &str) -> Vec<Token> {
    let mut list = Vec::<Token>::new();
    let mut buffer = String::new();
    let mut in_string = false;
    let mut in_calc = false;
    let mut in_script = false;
    let mut in_assertion = false;
    let mut lex_iter = code.chars().into_iter().enumerate().peekable();

    let flush_buffer = |b: &str| -> Result<Token, String> {
        match b {
            "table" => Ok(Token::new(TokenType::Table)),
            "relationship" => Ok(Token::new(TokenType::Relationship)),
            "value_list" => Ok(Token::new(TokenType::ValueList)),
            "table_occurence" => Ok(Token::new(TokenType::TableOccurence)),
            "script" => Ok(Token::new(TokenType::Script)),
            "test" => Ok(Token::new(TokenType::Test)),
            "end" => Ok(Token::new(TokenType::End)),
            _ => {
                let n = b.parse::<f64>();
                if n.is_ok() {
                    Ok(Token::with_value(TokenType::NumericLiteral, n.unwrap().to_string()))
                } else if !b.as_bytes()[0].is_ascii_digit() {
                    Ok(Token::with_value(TokenType::Identifier, b.to_string()))
                } else {
                    Err("Invalid Identifier".to_string())
                }
            }
        }
    };

    while let Some((idx, mut c)) = &lex_iter.next() {
        if c.is_whitespace() && buffer.is_empty() {
            continue;
        }

        while in_string == true {
            if c == '"' {
                list.push(Token { ttype: TokenType::String, text: buffer.to_string()});
                in_string = false;
                buffer.clear();
            }
            buffer.push(c);
            c = lex_iter.next().unwrap().1;
        }

        while in_script == true {
            if c == ']' {
                list.push(Token { ttype: TokenType::Script, text: buffer.to_string()});
                in_script = false;
                buffer.clear();
                break;
            }
            buffer.push(c);
            c = lex_iter.next().unwrap().1;
        }

        while in_calc == true {
            if c == '}' {
                list.push(Token { ttype: TokenType::Calculation, text: buffer.to_string()});
                in_calc = false;
                buffer.clear();
            }
            buffer.push(c);
            c = lex_iter.next().unwrap().1;
        }

        let mut scope = 1;
        while in_assertion == true {
            if scope == 0 {
                list.push(Token { ttype: TokenType::Assertion, text: buffer.to_string()});
                buffer.clear();
                while c.is_whitespace() {
                    c = lex_iter.next().unwrap().1;
                    continue;
                }
                if c == ',' {
                    c = lex_iter.next().unwrap().1;
                    while c.is_whitespace() {
                        c = lex_iter.next().unwrap().1;
                    }
                    if c == '(' {
                        scope = 1;
                    } else {
                        in_assertion = false;
                    }
                } else {
                    in_assertion = false;
                }
            } else if c == '(' {
                scope += 1;
            } else if c == ')' {
                scope -= 1;
            }
            buffer.push(c);
            c = lex_iter.next().unwrap().1;
        }

        let b = buffer.as_str();

        let tmp = match c {
            x if x.is_whitespace() => 
            {
                let mut ret: Vec<Token> = vec![];
                if !buffer.is_empty() {
                    let out = flush_buffer(b);
                    if out.is_ok() {
                        ret.push(out.unwrap());
                    buffer.clear();
                    }
                }
                ret

            },
            '=' =>  
            {
                let mut ret: Vec<Token> = vec![];
                if !buffer.is_empty() {
                    let out = flush_buffer(b);
                    if out.is_ok() {
                        ret.push(out.unwrap());
                    }
                    buffer.clear();
                }

                if let Some(n) = lex_iter.peek() {
                    if n.1 == '=' {
                        ret.push(Token::new(TokenType::EComparison));
                        lex_iter.next();
                    } else {
                        ret.push(Token::new(TokenType::Assign));
                    }
                }
                ret
            },
            '[' => 
            {
                let mut ret: Vec<Token> = vec![];
                if !buffer.is_empty() {
                    let out = flush_buffer(b);
                    if out.is_ok() {
                        ret.push(out.unwrap());
                    }
                    buffer.clear();
                }
                ret.push(Token::new(TokenType::OpenSquare));
                if list.last().unwrap().ttype != TokenType::FoundIn {
                    in_script = true;
                }
                ret
            },
            ']' => 
            {
                let mut ret: Vec<Token> = vec![];
                if !buffer.is_empty() {
                    let out = flush_buffer(b);
                    if out.is_ok() {
                        ret.push(out.unwrap());
                    }
                    buffer.clear();
                }
                ret.push(Token::new(TokenType::CloseSquare));
                ret
            },
            '(' => 
            {
                if list[list.len() - 2].ttype == TokenType::AssertionBlock {
                    in_assertion = true;       
                    buffer.push(c);
                    vec![]
                } else {
                    let mut ret: Vec<Token> = vec![];
                    if !buffer.is_empty() {
                        let out = flush_buffer(b);
                        if out.is_ok() {
                            ret.push(out.unwrap());
                        }
                        buffer.clear();
                    }
                    ret.push(Token::new(TokenType::OpenParen));
                    ret
                }

            },
            ')' => 
            {
                let mut ret: Vec<Token> = vec![];
                if !buffer.is_empty() {
                    let out = flush_buffer(b);
                    if out.is_ok() {
                        ret.push(out.unwrap());
                    }
                    buffer.clear();
                }
                ret.push(Token::new(TokenType::CloseParen));
                ret

            },
            '"' =>  
            {
                let mut ret: Vec<Token> = vec![];
                if !buffer.is_empty() {
                    let out = flush_buffer(b);
                    if out.is_ok() {
                        ret.push(out.unwrap());
                    }
                    buffer.clear();
                }
                in_string = true;
                ret
            },
            '!' =>  
            {
                let mut ret: Vec<Token> = vec![];
                if !buffer.is_empty() {
                    let out = flush_buffer(b);
                    if out.is_ok() {
                        ret.push(out.unwrap());
                    }
                    buffer.clear();
                }
                ret.push(Token::new(TokenType::Exclamation));
                ret
            },
            '{' => 
            {
                let mut ret: Vec<Token> = vec![];
                if !buffer.is_empty() {
                    let out = flush_buffer(b);
                    if out.is_ok() {
                        ret.push(out.unwrap());
                    }
                    buffer.clear();
                }
                ret.push(Token::new(TokenType::OpenCurly));
                in_calc = true;
                ret
            },
            '}' => {
                let mut ret: Vec<Token> = vec![];
                if !buffer.is_empty() {
                    let out = flush_buffer(b);
                    if out.is_ok() {
                        ret.push(out.unwrap());
                    }
                    buffer.clear();
                }
                ret.push(Token::new(TokenType::CloseCurly));
                ret
            },
            ';' => 
            {
                let mut ret: Vec<Token> = vec![];
                if !buffer.is_empty() {
                    let out = flush_buffer(b);
                    if out.is_ok() {
                        ret.push(out.unwrap());
                    }
                    buffer.clear();
                }
                ret.push(Token::new(TokenType::SemiColon));
                ret
            },
            ':' =>
            {
                let mut ret: Vec<Token> = vec![];
                if !buffer.is_empty() {
                    let out = flush_buffer(b);
                    if out.is_ok() {
                        ret.push(out.unwrap());
                    }
                    buffer.clear();
                }
                ret.push(Token::new(TokenType::Colon));
                ret
            }
            ',' =>  
            {
                let mut ret: Vec<Token> = vec![];
                if !buffer.is_empty() {
                    let out = flush_buffer(b);
                    if out.is_ok() {
                        ret.push(out.unwrap());
                    }
                    buffer.clear();
                }
                ret.push(Token::new(TokenType::Comma));
                ret
            }
            _ => 
            {
                buffer.push(c);
                vec![]
            }
        };
        if !tmp.is_empty() {
            for t in tmp {
                list.push(t);
            }
            buffer.clear();
        }
    }

    return list;

}
