
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    OpenParen,
    CloseParen,
    Identifier,
    NumericLiteral,
    String,
    SemiColon,
    Comma,
    OpenSquare,
    CloseSquare,
    Plus,
    Minus,
    Multiply,
    Divide,
    Ampersand,
    Eq,
    Neq,
    Gtq,
    Ltq,
    Gt,
    Lt,
}


#[derive(Debug, PartialEq, Eq, Clone)]
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

    pub fn with_value(t: TokenType, val: String) -> Self {
        Self {
            ttype: t,
            value: val,
        }
    }

}
