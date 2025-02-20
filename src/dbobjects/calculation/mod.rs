
pub mod context;
mod parser;
pub mod token;

use super::calculation::token::{Token, Function, GetArgument}; use super::calculation::context::CalculationContext; use super::calculation::parser::*;
use crate::util::encoding_util::fm_string_decrypt;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
enum Value {
    Number(f64),
    Text(String),
}

impl Value {
    fn as_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            Value::Text(s) => s.chars().filter(|c| c.is_numeric() || *c == '.').collect::<String>().parse().unwrap_or(0.0),
        }
    }

    fn as_text(&self) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::Text(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Calculation(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CalculationString(pub String);

impl Calculation {

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
            //println!("{}", self.0[ptr]);
            match self.0[ptr] {
                0x4 => {
                    result.push(Token::OpenParen);
                    ptr += 1;
                }
                0x5 => {
                    result.push(Token::CloseParen);
                    ptr += 1;
                }
                12 => {
                    ptr += 1;
                    if self.0[ptr] != 19 {  } // error
                    ptr += 1;
                    if self.0[ptr] != 1 {  } // error
                    ptr += 1;
                    if self.0[ptr] != 122 {  } // error
                    ptr += 1;
                    if self.0[ptr] != 0 { } // error
                    ptr += 1;
                }
                0x2d => {
                    result.push(Token::Function(Function::Abs));
                    ptr += 1;
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
                    let end = ptr + 20;
                    while ptr < end {
                        let cur = self.0[ptr];
                        if ptr == end - 11 {
                            result.push(Token::Number(cur as f64));
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
                    ptr += 1;
                    result.push(Token::Variable(String::from(&fm_string_decrypt(&self.0[ptr..ptr+len]))));
                    ptr += len;
                },
                0x25 => {
                    result.push(Token::Add);
                    ptr += 1;
                }
                0x26 => {
                    result.push(Token::Subtract);
                    ptr +=1;
                }
                0x27 => {
                    result.push(Token::Multiply);
                    ptr += 1;
                }
                0x28 => {
                    result.push(Token::Divide);
                    ptr +=1;
                },
                0x41 => {
                    result.push(Token::Less);
                    ptr +=1;
                }
                0x43 => {
                    result.push(Token::LessEqual);
                    ptr +=1;
                }
                0x44 => {
                    result.push(Token::Equal);
                    ptr +=1;
                }
                0x46 => {
                    result.push(Token::NotEqual);
                    ptr +=1;
                }
                0x47 => {
                    result.push(Token::GreaterEqual);
                    ptr +=1;
                }
                0x49 => {
                    result.push(Token::Greater);
                    ptr +=1;
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

    pub fn eval<T>(&self, ctx: &T) -> Result<String, String> where T: CalculationContext {
        let tokens = self.lex();
        let ast = parser::Parser::new(&tokens).parse();
        match ast.eval(ctx) {
            Ok(inner) => Ok(inner.as_text()),
            Err(e) => Err(e)
        }
    }
}


impl Expr {
    pub fn eval<T>(&self, ctx: &T) -> Result<Value, String> where T: CalculationContext {
        match self {
            Expr::Number(n) => Ok(Value::Text(n.to_string())), // Numbers are stored as text initially
            Expr::String(s) => Ok(Value::Text(s.to_string())),
            
            Expr::Variable(name) => ctx.get_var(name)
                .map(|val| Ok(Value::Text(val.clone())))
                .unwrap_or(Ok(Value::Text("".to_string()))),
            
            Expr::Global(name) => ctx.get_global_var(name)
                .map(|val| Ok(Value::Text(val.clone())))
                .unwrap_or(Ok(Value::Text("".to_string()))),

            Expr::FieldReference(reference) => {
                // Query database manager
                ctx.lookup_field(reference.clone())
                .map(|val| Value::Text(val))
                .ok_or_else(|| format!("Field not found: {:?}", reference))
            }

            Expr::BinaryOp { left, op, right } => {
                let left_val = left.eval(ctx)?;
                let right_val = right.eval(ctx)?;
                match op {
                    Token::Add => Ok(Value::Number(left_val.as_number() + right_val.as_number())),
                    Token::Subtract => Ok(Value::Number(left_val.as_number() - right_val.as_number())),
                    Token::Multiply => Ok(Value::Number(left_val.as_number() * right_val.as_number())),
                    Token::Divide => {
                        let denom = right_val.as_number();
                        if denom == 0.0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(Value::Number(left_val.as_number() / denom))
                        }
                    }
                    Token::Equal => {
                        Ok(Value::Text((left_val.as_number() == right_val.as_number()).to_string()))
                    },
                    Token::Concatenate => Ok(Value::Text(left_val.as_text() + &right_val.as_text())), // Concatenation
                    _ => Err("Unsupported binary operator".to_string()),
                }
            }

            Expr::UnaryOp { op, expr } => {
                let val = expr.eval(ctx)?;
                match op {
                    Token::Subtract => Ok(Value::Number(-val.as_number())),
                    _ => Err("Unsupported unary operator".to_string()),
                }
            }

            Expr::FunctionCall { name, args } => {
                let arg_vals: Result<Vec<Value>, _> = args.iter().map(|arg| arg.eval(ctx)).collect();
                let arg_vals = arg_vals?;
                match name.as_str() {
                    "Abs" => arg_vals.get(0).map(|v| Ok(Value::Number(v.as_number().abs()))).unwrap_or(Err("Missing argument for Abs".to_string())),
                    "UpperCase" => arg_vals.get(0).map(|v| Ok(Value::Text(v.as_text().to_uppercase()))).unwrap_or(Err("Missing argument for UpperCase".to_string())),
                    _ => Err(format!("Unknown function: {}", name)),
                }
            }

            Expr::Subscript { array, index } => {
                let array_val = array.eval(ctx)?.as_text(); // Assume arrays are stored as text
                let index_val = index.eval(ctx)?.as_number() as usize;
                array_val.chars().nth(index_val)
                    .map(|c| Ok(Value::Text(c.to_string())))
                    .unwrap_or(Err("Index out of bounds".to_string()))
            }
        }
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
                result.push(Token::Equal);
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
    use super::parser::*;
    use super::context::DummyContext;

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

    #[test]
    fn addition_eval() {
        let code = Calculation::from_text("3 + 12");
        println!("COde: {:?}", code);
        assert_eq!(code.eval(&DummyContext::new()).unwrap(), "15".to_string());
    }
}





