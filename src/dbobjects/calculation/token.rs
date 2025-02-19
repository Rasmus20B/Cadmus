
use crate::util::encoding_util::{fm_string_encrypt, put_path_int, get_path_int};
use crate::dbobjects::reference::FieldReference;

#[derive(Debug, PartialEq)]
pub enum GetArgument {
    CurrentTime,
    AccountName,
    DocumentsPath,
    DocumentsPathListing,
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum Token {
    Variable(String),
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
            Self::Space => {
                vec![12, 19, 1, 122, 0]
            }
            _ => todo!("{:?}", self)
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





