use super::token::{Location, Token, TokenType};

#[derive(Debug)]
pub enum LexErr {
    UnexpectedEOF,
}

#[derive(Clone, Copy)]
struct Cursor {
    line: u32,
    column: u32,
}

fn decode_buffer(buffer: &str, start: Location) -> Token {
    if let Ok(n) = buffer.parse::<usize>() {
        return Token::with_value(TokenType::IntegerLiteral, start, n.to_string())
    }

    if buffer.split("::").collect::<Vec<_>>().len() == 2 {
        Token::with_value(TokenType::FieldReference, start, buffer.to_string());
    }
    match buffer {
        "calculated_val" => {
            Token::new(TokenType::CalculatedVal, start) 
        }
        "datatype" => {
            Token::new(TokenType::Datatype, start) 
        }
        "do_not_replace" => {
            Token::new(TokenType::DoNotReplace, start)
       }
        "false" => {
            Token::new(TokenType::False, start) 
        }
        "field" => {
            Token::new(TokenType::Field, start) 
        },
        "generate" => {
            Token::new(TokenType::Generate, start)
        },
        "increment" => {
            Token::new(TokenType::Increment, start)
        }
        "next" => {
            Token::new(TokenType::Next, start)
        }
        "Number" => {
            Token::new(TokenType::Number, start) 
        }
        "on_creation" => {
            Token::new(TokenType::OnCreation, start) 
        }
        "on_commit" => {
            Token::new(TokenType::OnCommit, start) 
        }
        "relation" => { 
            Token::new(TokenType::Relation, start) 
        },
        "required" => {
            Token::new(TokenType::Required, start) 
        }
        "script" => {
            Token::new(TokenType::Script, start)
        }
        "serial" => {
            Token::new(TokenType::Serial, start) 
        }
        "table" => { 
            Token::new(TokenType::Table, start) 
        },
        "table_occurrence" => { 
            Token::new(TokenType::TableOccurrence, start) 
        },
        "test" => {
            Token::new(TokenType::Test, start) 
        }
        "true" => {
            Token::new(TokenType::True, start) 
        }
        "unique" => {
            Token::new(TokenType::Unique, start) 
        }
        "validation_message" => {
            Token::new(TokenType::ValidationMessage, start) 
        }
        "value_list" => {
            Token::new(TokenType::ValueList, start)
        }
        _ => { Token::with_value(TokenType::Identifier, start, buffer.to_string())},
    }

}

pub fn lex(code: &str) -> Result<Vec<Token>, LexErr> {
    let mut tokens = vec![];

    let mut cursor = Location { line: 0, column: 0 };

    let mut lex_iter = code.chars().peekable();
    let mut buffer = String::new();
    let mut token_start = cursor;


    let mut in_script = false;

    while let Some(c) = lex_iter.next() {
        if c.is_whitespace() && buffer.is_empty() {
            if c == '\n'{
                cursor.line += 1;
                cursor.column = 0;
            } else {
                cursor.column += 1;
            }
            continue;
        }

        match c {
            '\n' => {
                if !buffer.is_empty() {
                    tokens.push(decode_buffer(&buffer, token_start));
                    buffer.clear();
                }
                cursor.line += 1;
                cursor.column = 0;
            },
            ' ' => {
                if !buffer.is_empty() {
                    let kw = decode_buffer(&buffer, token_start);
                    if [TokenType::Script, TokenType::Test].contains(&kw.ttype)  {
                        in_script = true;
                    }
                    tokens.push(kw);
                    buffer.clear();
                }
            }
            '%' => {
                if !buffer.is_empty() {
                    tokens.push(decode_buffer(&buffer, token_start));
                    buffer.clear();
                }
                cursor.column += 1;
                if let Some(number) = lex_iter.next() {
                    tokens.push(Token::with_value(TokenType::ObjectNumber, cursor, number.to_string()));
                } else {
                    return Err(LexErr::UnexpectedEOF)
                }
            },
            '{' => {
                if !buffer.is_empty() {
                    tokens.push(decode_buffer(&buffer, token_start));
                    buffer.clear();
                }
                if in_script {
                    let mut depth = 1;
                    while let Some(c) = lex_iter.next() {
                        match c {
                            '}' => {
                                buffer.push(c);
                                depth -= 1;
                            }
                            '{' => {
                                depth += 1;
                                buffer.push(c);
                            }
                            _ => {
                                buffer.push(c);
                            }
                        }
                        if depth == 0 {
                            break;
                        }
                    }
                    if depth != 0 {
                        return Err(LexErr::UnexpectedEOF)
                    }
                    // Pop the script close brace at the end.
                    buffer.pop();
                tokens.push(Token::new(TokenType::OpenBrace, cursor));
                tokens.push(Token::with_value(TokenType::ScriptContent, cursor, buffer.clone()));
                tokens.push(Token::new(TokenType::CloseBrace, cursor));
                buffer.clear();
                } else {
                    tokens.push(Token::new(TokenType::OpenBrace, cursor));
                }
            },
            '}' => {
                if !buffer.is_empty() {
                    tokens.push(decode_buffer(&buffer, token_start));
                    buffer.clear();
                }
                tokens.push(Token::new(TokenType::CloseBrace, cursor));
            },
            '=' => {
                if !buffer.is_empty() {
                    tokens.push(decode_buffer(&buffer, token_start));
                    buffer.clear();
                }
                tokens.push(Token::new(TokenType::Assignment, cursor));
            },
            ',' => {
                if !buffer.is_empty() {
                    tokens.push(decode_buffer(&buffer, token_start));
                    buffer.clear();
                }
                tokens.push(Token::new(TokenType::Comma, cursor));
            }
            '[' => {
                if !buffer.is_empty() {
                    tokens.push(decode_buffer(&buffer, token_start));
                    buffer.clear();
                }
                for c in lex_iter.by_ref() {
                    if c == ']' {
                        tokens.push(Token::with_value(TokenType::Calculation, cursor, buffer.clone()));
                        cursor.column += buffer.len() as u32;
                        buffer.clear();
                        break;
                    } else {
                        buffer.push(c);
                    }
                }
            }
            '"' => {
                if !buffer.is_empty() {
                    tokens.push(decode_buffer(&buffer, token_start));
                    buffer.clear();
                }
                for c in lex_iter.by_ref() {
                    if c == '"' {
                        tokens.push(Token::with_value(TokenType::String, cursor, buffer.clone()));
                        cursor.column += buffer.len() as u32;
                        buffer.clear();
                        break;
                    } else {
                        buffer.push(c);
                    }
                }
            }
            ':' => {
                if !buffer.is_empty() {
                    tokens.push(decode_buffer(&buffer, token_start));
                    buffer.clear();
                }
                tokens.push(Token::new(TokenType::Colon, cursor));
            }
            '!' => {
                if !buffer.is_empty() {
                    tokens.push(decode_buffer(&buffer, token_start));
                    buffer.clear();
                }
                tokens.push(Token::new(TokenType::Exclamation, cursor));
            }
            '/' => {
                if !buffer.is_empty() {
                    tokens.push(decode_buffer(&buffer, token_start));
                    buffer.clear();
                }
                let next = lex_iter.next();
                if next == Some('/') {
                    while let Some(c) = lex_iter.next() {
                        if c == '\n' {
                            cursor.line += 1;
                            cursor.column = 0;
                            break;
                        }
                    }
                    continue;
                }

            }
            _ => {
                if buffer.is_empty() {
                    token_start = cursor;
                }
                buffer.push(c)
            }
        };
        cursor.column += 1;
    }

    tokens.push(Token::new(TokenType::EOF, cursor));
    Ok(tokens)
}


#[cfg(test)]
mod tests {
    use crate::cadlang::token::{Location, Token, TokenType};

    use super::lex;

    #[test]
    fn lex_test() {
        let code = "
table %1 Person {
    field %1 id = {
        datatype = Number,
        // This is a comment
        required =true, // This is also a comment
        unique= true,
        calculated_val = [get(uuid)],
        validation_message = \"Invalid ID chosen.\",
    }
}
            ";

        let expected = vec![
            Token::new(TokenType::Table, Location { line: 1, column: 0 }),
            Token::with_value(TokenType::ObjectNumber, Location { line: 1, column: 7 }, "1".to_string()),
            Token::with_value(TokenType::Identifier, Location { line: 1, column: 9 }, "Person".to_string()),
            Token::new(TokenType::OpenBrace, Location { line: 1, column: 16 }),
            Token::new(TokenType::Field, Location { line: 2, column: 4 }),
            Token::with_value(TokenType::ObjectNumber, Location { line: 2, column: 11 }, "1".to_string()),
            Token::with_value(TokenType::Identifier, Location { line: 2, column: 13 }, "id".to_string()),
            Token::new(TokenType::Assignment, Location { line: 2, column: 16 }),
            Token::new(TokenType::OpenBrace, Location { line: 2, column: 18 }),

            Token::new(TokenType::Datatype, Location { line: 3, column: 8 }),
            Token::new(TokenType::Assignment, Location { line: 3, column: 17 }),
            Token::new(TokenType::Number, Location { line: 3, column: 19 }),
            Token::new(TokenType::Comma, Location { line: 3, column: 25 }),

            Token::new(TokenType::Required, Location { line: 5, column: 8 }),
            Token::new(TokenType::Assignment, Location { line: 5, column: 17 }),
            Token::new(TokenType::True, Location { line: 5, column: 18 }),
            Token::new(TokenType::Comma, Location { line: 5, column: 22 }),

            Token::new(TokenType::Unique, Location { line: 6, column: 8 }),
            Token::new(TokenType::Assignment, Location { line: 6, column: 14 }),
            Token::new(TokenType::True, Location { line: 6, column: 16 }),
            Token::new(TokenType::Comma, Location { line: 6, column: 20 }),

            Token::new(TokenType::CalculatedVal, Location { line: 7, column: 8 }),
            Token::new(TokenType::Assignment, Location { line: 7, column: 23 }),
            Token::with_value(TokenType::Calculation, Location { line: 7, column: 25 }, "get(uuid)".to_string()),
            Token::new(TokenType::Comma, Location { line: 7, column: 35 }),

            Token::new(TokenType::ValidationMessage, Location { line: 8, column: 8 }),
            Token::new(TokenType::Assignment, Location { line: 8, column: 27 }),
            Token::with_value(TokenType::String, Location { line: 8, column: 29 }, "Invalid ID chosen.".to_string()),
            Token::new(TokenType::Comma, Location { line: 8, column: 48 }),

            Token::new(TokenType::CloseBrace, Location { line: 9, column: 4 }),
            Token::new(TokenType::CloseBrace, Location { line: 10, column: 0 }),
        ];

        let lexed = lex(&code).expect("Unable to lex code.");

        for pair in expected.iter().zip(lexed) {
            // println!("{:?} == {:?}", pair.0, pair.1);
            assert_eq!(*pair.0, pair.1);
        }
    }

}



