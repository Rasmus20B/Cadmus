
use crate::util::encoding_util::fm_string_encrypt;
use crate::dbobjects::reference::FieldReference;

#[derive(Debug, PartialEq)]
pub enum Token {
    Variable(String),
    Number(f64),
    String(String),
    Identifier(String),
    FieldReference(String, String),
    ResolvedFieldReference(FieldReference),

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
    fn encode(&self) -> Vec<u8> {
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
                todo!()
            }
            _ => todo!()
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





