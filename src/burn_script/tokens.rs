
#[derive(Debug, PartialEq)]
pub enum TokenType {
    Define,
    Identifier,
    OpenParen,
    CloseParen,
    SemiColon,
    Comma,
    OpenBracket,
    CloseBracket,
    Let,
    Assign,
    Plus,
    Minus,
    Multiply,
    Divide,
    Ampersand,
    Loop,
    If,
    Elif,
    Else,
    Eq,
    Neq,
    Geq,
    Leq,
    Gt,
    Lt,
    NumericLiteral,
    String,
    Argument,
}

#[derive(PartialEq, Debug)]
pub struct Token {
    pub ttype: TokenType,
    pub value: String,
}

impl Token {
    pub fn new(t: TokenType) -> Self {
        Self {
            ttype: t,
            value: String::new(),
        }
    }
    pub fn with_value(t:TokenType, val: &str) -> Self {
        Self {
            ttype: t,
            value: val.to_string(),
        }
    }
}
