
#[derive(Debug, PartialEq)]
pub enum TokenType {
    Table,
    Relationship,
    ValueList,
    Script,
    Test,
    TableOccurence,
    AssertionBlock,
    Assertion,
    Identifier,
    Assign,
    DataType,
    SemiColon,
    Colon,
    Comma,
    OpenCurly,
    CloseCurly,
    OpenSquare,
    CloseSquare,
    OpenParen,
    CloseParen,
    OpenQuote,
    CloseQuote,
    Exclamation,
    NewLine,
    Calculation, /* Anything found within a calculation specifier will be compiled in a different layer */
    End,
    EComparison,
    NEComparison,
    LComparison,
    LEComparison,
    GComparison,
    GEComparison,
    CComparison,
    Unique,
    Required,
    Existing,
    FoundIn,
    Context,
    String,
    NumericLiteral,
    EOF,
}

pub struct Token {
    pub ttype: TokenType,
    pub text: String,
}

impl Token {
    pub fn new(t: TokenType) -> Self {
        Self { ttype: t, text: String::new() }
    }

    pub fn with_value(t: TokenType, val: String) -> Self {
        Self { ttype: t, text: val }
    }
}
