use core::fmt;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
    // Top level objects
    Table,
    TableOccurrence,
    Relation,
    ValueList,
    Layout,
    Script,
    Test,
    Extern,
    // Second level objects
    Field,
    // thid level objects
    AutoIncrement,
    AutoEntry,
    Datatype,

    Identifier,
    ObjectNumber,
    Assignment,
    Calculation,
    IntegerLiteral,
    String,

    // Auto Entry Tab
    Creation,
    Modification,
    Serial,
    Generate,
    OnCreation,
    OnCommit,
    Next,
    Increment,
    LastVisited,
    Data,
    CalculatedVal,
    DoNotReplace,
    Lookup,

    // Lookup 
    StartTable,
    RelatedTable,

    // Validation Tab
    Validate,
    Always,
    OnEntry,
    AllowOverride,

    // Validate Require tab
    StrictDataType,
    NotEmpty,
    Required,
    Unique,
    MemberOf,
    Range,
    ValidationCalc,
    MaxChars,
    ValidationMessage,

    // Storage Tab
    Global,
    Repetitions,

    // Valuelist attributes
    From,
    Sort,
    FirstField,
    SecondField,

    Number,
    Text,
    Date,
    ScopeResolution,
    ScriptContent,

    True,
    False,

    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Cartesian,

    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenSquare,
    CloseSquare,
    Comma,
    Colon,
    Exclamation,
    EOF,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assignment => write!(f, "="),
            Self::OpenBrace => write!(f, "{{"),
            Self::CloseBrace => write!(f, "}}"),
            Self::OpenParen => write!(f, "("),
            Self::CloseParen => write!(f, ")"),
            Self::ScopeResolution => write!(f, "Field Reference"),
            _ => write!(f, "{:?}", self)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub ttype: TokenType,
    pub value: String,
    pub location: Location,
}

impl Token {
    pub fn new(ttype_: TokenType, location_: Location) -> Self {
        Self {
            ttype: ttype_,
            value: String::new(),
            location: location_,
        }
    }

    pub fn with_value(ttype_: TokenType, location_: Location, value_: String) -> Self {
        Self {
            ttype: ttype_,
            value: value_,
            location: location_,
        }
    }
}
