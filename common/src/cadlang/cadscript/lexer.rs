
use super::token::*;

pub fn lex(code: &str) -> Vec<TokenVal> {
    let mut tokens = vec![];
    let mut lex_iter = code.chars().peekable();
    let mut buffer = String::new();
    while let Some(mut c) = lex_iter.next() {
        match c {
            c if c.is_alphabetic() || c == '$' => {
                buffer.push(c);
                let ch = lex_iter.peek().unwrap();
                if ch.is_alphanumeric() || *ch == '_' || *ch == ':' {
                    buffer.push(*ch)
                } else {
                    break;
                }
                while let Some(c) = lex_iter.next() {
                    let ch = lex_iter.peek().unwrap();
                    if ch.is_alphanumeric() || *ch == '_' || *ch == ':' {
                        buffer.push(*ch)
                    } else {
                        break;
                    }
                }

                if buffer.ends_with("=") {
                    buffer.pop();
                    tokens.push(TokenVal::ArgLabel(buffer.clone()));
                    tokens.push(TokenVal::Assignment);
                } else if let [table, field] = buffer.split("::").collect::<Vec<_>>()[..] {
                    println!("GETS HERE BRUH");
                    tokens.push(TokenVal::FieldReference(table.to_string(), field.to_string()));
                } else {
                    tokens.push(TokenVal::Identifier(buffer.clone()));
                }

                buffer.clear();
            }
            '$' => {
                buffer.push(c);
                while let Some(c) = lex_iter.next() {
                    if [' ', ',', ')'].contains(&c) {
                        break;
                    }
                }

                if buffer.chars().nth(1).unwrap() == '$' {
                    tokens.push(TokenVal::Global(buffer.clone()));
                } else {
                    tokens.push(TokenVal::Variable(buffer.clone()));
                }
                buffer.clear();
            }

            '|' => {
                let mut in_string = false;
                let mut escaped = false;
                while let Some(c) = lex_iter.next() {
                    if c == '\"' {
                        if in_string && escaped {
                            escaped = false;
                        } else if in_string {
                            in_string = false;
                        } else {
                            in_string = true;
                        }
                        buffer.push(c);
                    } else if c == '\\' {
                        if in_string {
                            if escaped {
                                escaped = false;
                            } else {
                                escaped = true;
                            }
                        }
                        buffer.push(c);
                    } else if c == '|' {
                        if !in_string {
                            break;
                        }
                    } else {
                        buffer.push(c);
                    }
                }

                tokens.push(TokenVal::CalculationArg(buffer.clone()));
                buffer.clear();
            }

            '\"' => {
                let mut escaped = false;
                while let Some(c) = lex_iter.next() {
                    if c == '\"' && !escaped {
                        break;
                    } else if c == '\\' && !escaped {
                        escaped = true;
                        buffer.push(c);
                    } else {
                        escaped = false;
                        buffer.push(c);
                    }
                }

                tokens.push(TokenVal::StringArg(buffer.clone()));
                buffer.clear();
            }
            ';' => tokens.push(TokenVal::SemiColon),
            '{' => tokens.push(TokenVal::OpenBrace),
            '}' => tokens.push(TokenVal::CloseBrace),
            '(' => tokens.push(TokenVal::OpenParen),
            ')' => tokens.push(TokenVal::CloseParen),
            ',' => tokens.push(TokenVal::Comma),
            _ => {}
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_lex() {
        let code = "set_variable($x, |0|);
                    go_to_layout(Person);
                    loop {
                      exit_loop_if(|$x == 10|);
                      new_record_request();
                      set_variable($x, |$x + 1|);
                      set_field(Person::name, |$x|);
                    }";

       let expected = vec![
           TokenVal::Identifier(String::from("set_variable")),
           TokenVal::OpenParen,
           TokenVal::Identifier(String::from("$x")),
           TokenVal::Comma,
           TokenVal::CalculationArg(String::from("0")),
           TokenVal::CloseParen,
           TokenVal::SemiColon,
           
           TokenVal::Identifier(String::from("go_to_layout")),
           TokenVal::OpenParen,
           TokenVal::Identifier(String::from("Person")),
           TokenVal::CloseParen,
           TokenVal::SemiColon,

           TokenVal::Identifier(String::from("loop")),
           TokenVal::OpenBrace,

           TokenVal::Identifier(String::from("exit_loop_if")),
           TokenVal::OpenParen,
           TokenVal::CalculationArg(String::from("$x == 10")),
           TokenVal::CloseParen,
           TokenVal::SemiColon,

           TokenVal::Identifier(String::from("new_record_request")),
           TokenVal::OpenParen,
           TokenVal::CloseParen,
           TokenVal::SemiColon,

           TokenVal::Identifier(String::from("set_variable")),
           TokenVal::OpenParen,
           TokenVal::Identifier(String::from("$x")),
           TokenVal::Comma,
           TokenVal::CalculationArg(String::from("$x + 1")),
           TokenVal::CloseParen,
           TokenVal::SemiColon,

           TokenVal::Identifier(String::from("set_field")),
           TokenVal::OpenParen,
           TokenVal::FieldReference(String::from("Person"), String::from("name")),
           TokenVal::Comma,
           TokenVal::CalculationArg(String::from("$x")),
           TokenVal::CloseParen,
           TokenVal::SemiColon,

           TokenVal::CloseBrace,
       ];

       assert_eq!(lex(code), expected);

    }
}
