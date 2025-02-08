

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Token {
    Identifier(String),
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
