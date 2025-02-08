
use super::token::Token;

pub fn lex(code: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let mut lex_iter = code.chars().peekable();
    let mut buffer = String::new();
    while let Some(mut c) = lex_iter.next() {
        match c {
            c if c.is_alphabetic() || c == '$' => {
                buffer.push(c);
                let ch = lex_iter.peek().unwrap();
                if ch.is_alphanumeric() || *ch == '_' {
                    buffer.push(*ch)
                } else {
                    break;
                }
                while let Some(c) = lex_iter.next() {
                    let ch = lex_iter.peek().unwrap();
                    if ch.is_alphanumeric() || *ch == '_' {
                        buffer.push(*ch)
                    } else {
                        break;
                    }
                }

                if buffer.ends_with("=") {
                    buffer.pop();
                    tokens.push(Token::ArgLabel(buffer.clone()));
                    tokens.push(Token::Assignment);
                } else {
                    tokens.push(Token::Identifier(buffer.clone()));
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

                tokens.push(Token::CalculationArg(buffer.clone()));
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

                tokens.push(Token::StringArg(buffer.clone()));
                buffer.clear();
            }
            '{' => tokens.push(Token::OpenBrace),
            '}' => tokens.push(Token::CloseBrace),
            '(' => tokens.push(Token::OpenParen),
            ')' => tokens.push(Token::CloseParen),
            ',' => tokens.push(Token::Comma),
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
                    go_to_layout(\"Person\");
                    loop {
                      exit_loop_if(|$x == 10|);
                      new_record_request();
                      set_variable($x, |$x + 1|);
                    }";

       let expected = vec![
           Token::Identifier(String::from("set_variable")),
           Token::OpenParen,
           Token::Identifier(String::from("$x")),
           Token::Comma,
           Token::CalculationArg(String::from("0")),
           Token::CloseParen,
           
           Token::Identifier(String::from("go_to_layout")),
           Token::OpenParen,
           Token::StringArg(String::from("Person")),
           Token::CloseParen,

           Token::Identifier(String::from("loop")),
           Token::OpenBrace,

           Token::Identifier(String::from("exit_loop_if")),
           Token::OpenParen,
           Token::CalculationArg(String::from("$x == 10")),
           Token::CloseParen,

           Token::Identifier(String::from("new_record_request")),
           Token::OpenParen,
           Token::CloseParen,

           Token::Identifier(String::from("set_variable")),
           Token::OpenParen,
           Token::Identifier(String::from("$x")),
           Token::Comma,
           Token::CalculationArg(String::from("$x + 1")),
           Token::CloseParen,

           Token::CloseBrace,
       ];

       assert_eq!(lex(code), expected);

    }
}
