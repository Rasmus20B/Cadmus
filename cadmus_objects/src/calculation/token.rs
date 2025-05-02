
use cadmus_util::encoding_util::{fm_string_encrypt, put_path_int, get_path_int};
use crate::reference::FieldReference;

#[derive(Debug, PartialEq, Clone)]
pub enum GetArgument {
    CurrentTime,
    AccountName,
    DocumentsPath,
    DocumentsPathListing,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Function {
    Char,
    Abs,
    Cos,
    Sin,
    Tan,
    Acos,
    Asin,
    Atan,
    Get(GetArgument),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Variable,
    Global,
    Number,
    String,
    Identifier,
    FieldReference,
    ResolvedFieldReference,
    Function,

    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    Add,
    Multiply,
    Subtract,
    Divide,
    Concatenate,

    Negate,

    OpenParen,
    CloseParen,

    OpenSquare,
    CloseSquare,

    SemiColon,

    Space,
}

impl From<&Token> for TokenKind {
    fn from(token: &Token) -> TokenKind {
        match token {
            Token::Variable(..) => TokenKind::Variable,
            Token::Number(..) => TokenKind::Number,
            Token::String(..) => TokenKind::String,
            Token::Identifier(..) => TokenKind::Identifier,
            Token::FieldReference(..) => TokenKind::FieldReference,
            Token::ResolvedFieldReference(..) => TokenKind::ResolvedFieldReference,
            Token::Function(..) => TokenKind::Function,

            Token::Equal => TokenKind::Equal,
            Token::NotEqual => TokenKind::NotEqual,
            Token::Less => TokenKind::Less,
            Token::LessEqual => TokenKind::LessEqual,
            Token::Greater => TokenKind::Greater,
            Token::GreaterEqual => TokenKind::GreaterEqual,

            Token::Add => TokenKind::Add,
            Token::Multiply => TokenKind::Multiply,
            Token::Subtract => TokenKind::Subtract,
            Token::Divide => TokenKind::Divide,
            Token::Concatenate => TokenKind::Concatenate,

            Token::Negate => TokenKind::Negate,

            Token::OpenParen => TokenKind::OpenParen,
            Token::CloseParen => TokenKind::CloseParen,

            Token::OpenSquare => TokenKind::OpenSquare,
            Token::CloseSquare => TokenKind::CloseSquare,

            Token::SemiColon => TokenKind::SemiColon,
            Token::Space => TokenKind::Space,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Variable(String),
    Global(String),
    Number(f64),
    String(String),
    Identifier(String),
    FieldReference(String, String),
    ResolvedFieldReference(FieldReference),
    Function(Function),

    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    Add,
    Multiply,
    Subtract,
    Divide,
    Concatenate,

    Negate,

    OpenParen,
    CloseParen,

    OpenSquare,
    CloseSquare,

    SemiColon,

    Space,
}

impl Token {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::Variable(ident) => {
                let mut result = vec![];
                result.push(0x1A);
                result.push(ident.len() as u8);
                result.extend(fm_string_encrypt(ident));
                result
            }
            Self::Add => vec![0x25],
            Self::Subtract | Self::Negate => vec![0x26],
            Self::Multiply => vec![0x27],
            Self::Divide => vec![0x28],
            Self::Concatenate => vec![0x50],
            Self::ResolvedFieldReference(reference) => {
                let (ds, table, field) = (reference.data_source, reference.table_occurrence_id, reference.field_id);
                vec![0x16, 0x4, 0x4, 0x3, 0xD0, 0x0, table as u8, 0x2, 0x1, field as u8, 0x0, 0x1]
            }
            Self::Number(n) => {
                vec![16, 2, 0, 1, 0, 16, 0, 0, 0, (*n) as u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32]
            }
            Self::Equal => {
                vec![0x44]
            }
            Self::Space => {
                vec![12, 19, 1, 122, 0]
            }
            _ => todo!("{:?}", self)
        }
    }

    pub fn kind(&self) -> TokenKind {
        match self {
            Token::Variable(..) => TokenKind::Variable,
            Token::Global(..) => TokenKind::Global,
            Token::Number(..) => TokenKind::Number,
            Token::String(..) => TokenKind::String,
            Token::Identifier(..) => TokenKind::Identifier,
            Token::FieldReference(..) => TokenKind::FieldReference,
            Token::ResolvedFieldReference(..) => TokenKind::ResolvedFieldReference,
            Token::Function(..) => TokenKind::Function,

            Token::Equal => TokenKind::Equal,
            Token::NotEqual => TokenKind::NotEqual,
            Token::Less => TokenKind::Less,
            Token::LessEqual => TokenKind::LessEqual,
            Token::Greater => TokenKind::Greater,
            Token::GreaterEqual => TokenKind::GreaterEqual,

            Token::Add => TokenKind::Add,
            Token::Multiply => TokenKind::Multiply,
            Token::Subtract => TokenKind::Subtract,
            Token::Divide => TokenKind::Divide,
            Token::Concatenate => TokenKind::Concatenate,

            Token::Negate => TokenKind::Negate,

            Token::OpenParen => TokenKind::OpenParen,
            Token::CloseParen => TokenKind::CloseParen,

            Token::OpenSquare => TokenKind::OpenSquare,
            Token::CloseSquare => TokenKind::CloseSquare,

            Token::SemiColon => TokenKind::SemiColon,
            Token::Space => TokenKind::Space,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn token_encoding() {
        assert_eq!(Token::encode(&Token::Variable(String::from("$x"))),
            vec![0x1A, 0x02, 0x7E, 0x22]);
    }
}





