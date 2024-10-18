
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
    // Top level objects
    Table,
    TableOccurrence,
    Relation,
    ValueList,
    Script,
    Test,
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
    Calculated,
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

    Number,
    Text,
    Date,

    True,
    False,

    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenSquare,
    CloseSquare,
    Comma,
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
