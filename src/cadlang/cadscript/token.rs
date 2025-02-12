

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TokenVal {
    Identifier(String),
    FieldReference(String, String),
    Variable(String),
    CalculationArg(String),
    StringArg(String),
    NumberArg(f64),
    KeywordArg(String),
    ArgLabel(String),

    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,

    Assignment,

    Comma,
    SemiColon,
}

impl TokenVal {
    pub fn get_value(&self) -> Option<String> {
        match self {
            TokenVal::Identifier(val) => Some(val.to_string()),
            TokenVal::CalculationArg(val) => Some(val.to_string()),
            TokenVal::StringArg(val) => Some(val.to_string()),
            TokenVal::NumberArg(val) => Some(val.to_string()),
            TokenVal::KeywordArg(val) => Some(val.to_string()),
            TokenVal::ArgLabel(val) => Some(val.to_string()),

            _ => None,
        }
    }
}

pub struct Token {
    pub value: TokenVal,
    pub location: (usize, usize),
}
