
use std::iter::Peekable;
use std::slice::Iter;

use super::token::*;

use crate::reference::FieldReference;


#[derive(Debug, PartialEq)]
pub enum Expr {
    String(String),
    Number(f64),
    Variable(String),
    Global(String),
    FieldReference(FieldReference),
    BinaryOp {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    UnaryOp {
        op: Token,
        expr: Box<Expr>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    Subscript {
        array: Box<Expr>,
        index: Box<Expr>,
    },
}

pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: tokens.iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Expr {
        self.parse_expression()
    }

    fn parse_expression(&mut self) -> Expr {
        self.parse_concatenation()
    }

    fn parse_concatenation(&mut self) -> Expr {
        let mut expr = self.parse_comparison();
        while let Some(Token::Concatenate) = self.tokens.peek() {
            self.tokens.next();
            let right = self.parse_comparison();
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: Token::Concatenate,
                right: Box::new(right),
            };
        }
        expr
    }


     fn parse_comparison(&mut self) -> Expr {
        let mut expr = self.parse_term();
        while let Some(op) = self.tokens.peek() {
            match op {
                Token::Equal | Token::NotEqual | Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual => {
                    let op = self.tokens.next().unwrap().clone();
                    let right = self.parse_term();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_term(&mut self) -> Expr {
        let mut expr = self.parse_factor();
        while let Some(op) = self.tokens.peek() {
            match op {
                Token::Add | Token::Subtract => {
                    let op = self.tokens.next().unwrap().clone();
                    let right = self.parse_factor();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_factor(&mut self) -> Expr {
        let mut expr = self.parse_unary();
        while let Some(op) = self.tokens.peek() {
            match op {
                Token::Multiply | Token::Divide => {
                    let op = self.tokens.next().unwrap().clone();
                    let right = self.parse_unary();
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_unary(&mut self) -> Expr {
        if let Some(Token::Subtract) = self.tokens.peek() {
            let op = self.tokens.next().unwrap().clone();
            let expr = self.parse_primary();
            return Expr::UnaryOp {
                op,
                expr: Box::new(expr),
            };
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Expr {
        match self.tokens.next() {
            Some(Token::Number(n)) => Expr::Number(*n),
            Some(Token::String(s)) => Expr::String(s.to_string()),
            Some(Token::Variable(v)) => Expr::Variable(v.to_string()),
            Some(Token::Global(g)) => Expr::Global(g.to_string()),
            Some(Token::Identifier(name)) => {
                if let Some(Token::OpenParen) = self.tokens.peek() {
                    self.tokens.next(); // Consume '('
                    let mut args = Vec::new();
                    while let Some(token) = self.tokens.peek() {
                        if matches!(token, Token::CloseParen) {
                            break;
                        }
                        args.push(self.parse_expression());
                        if matches!(self.tokens.peek(), Some(Token::SemiColon)) {
                            self.tokens.next(); // Consume ';'
                        }
                    }
                    self.tokens.next(); // Consume ')'
                    Expr::FunctionCall {
                        name: name.clone(),
                        args,
                    }
                } else {
                    // Error
                    todo!()
                }
            }
            Some(Token::OpenParen) => {
                let expr = self.parse_expression();
                self.tokens.next(); // Consume ')'
                expr
            }
            Some(Token::ResolvedFieldReference(reference)) => Expr::FieldReference(reference.clone()),
            Some(Token::OpenSquare) => {
                let array = self.parse_expression();
                self.tokens.next(); // Consume '['
                let index = self.parse_expression();
                self.tokens.next(); // Consume ']'
                Expr::Subscript {
                    array: Box::new(array),
                    index: Box::new(index),
                }
            }
            Some(_) | None => panic!("Unexpected token or EOF"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_addition() {
        let tokens: Vec<Token> = vec![
            Token::Number(6.0),
            Token::Add,
            Token::Number(1.0),
        ];
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse();
        if let Expr::BinaryOp { ref left, ref op, ref right } = ast {
            assert_eq!(**left, Expr::Number(6.0));
            assert_eq!(*op, Token::Add);
            assert_eq!(**right, Expr::Number(1.0));
        }
    }

    #[test]
    fn group_arithmetic() {
        let tokens: Vec<Token> = vec![
            Token::OpenParen,
            Token::Number(3.0),
            Token::Add,
            Token::Number(2.0),
            Token::CloseParen,
            Token::Multiply,
            Token::Number(4.0),
        ];
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse();
        assert_eq!(Expr::BinaryOp { 
            left: Box::new(Expr::BinaryOp { 
                left: Box::new(Expr::Number(3.0)), 
                op: Token::Add, 
                right: Box::new(Expr::Number(2.0)) 
            }), 
            op: Token::Multiply, 
            right: Box::new(Expr::Number(4.0))
            }
        , ast);
    }

    #[test]
    fn string_concat() {
        let tokens: Vec<Token> = vec![
            Token::String("FileMaker".to_string()),
            Token::Concatenate,
            Token::String(" Testing".to_string()),
        ];
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse();
        match ast {
            Expr::BinaryOp { ref left, ref op, ref right } => {
                assert_eq!(**left, Expr::String("FileMaker".to_string())); 
                assert_eq!(*op, Token::Concatenate);
                assert_eq!(**right, Expr::String(" Testing".to_string()));
            }
            _ => { unreachable!() }
        }
    }
}
