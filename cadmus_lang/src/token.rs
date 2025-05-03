use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub struct SourceLoc {
    line: u32,
    column: u32,
}

impl SourceLoc {
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub token_val: TokenValue,
    pub source_loc: SourceLoc,
}

impl Token {
    pub fn new(token_val: TokenValue, source_loc: SourceLoc) -> Self {
        Self {
            token_val,
            source_loc,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenValue {
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

    Identifier(String),
    ObjectNumber(u32),
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

pub struct KeywordEntry {
    pub text: &'static str,
    pub val: TokenValue,
}

pub const KEYWORD_MAP: [KeywordEntry; 8] = [
    KeywordEntry {
        text: "extern",
        val: TokenValue::Extern,
    },
    KeywordEntry {
        text: "layout",
        val: TokenValue::Layout,
    },
    KeywordEntry {
        text: "relation",
        val: TokenValue::Relation,
    },
    KeywordEntry {
        text: "table",
        val: TokenValue::Table,
    },
    KeywordEntry {
        text: "table_occurrence",
        val: TokenValue::TableOccurrence,
    },
    KeywordEntry {
        text: "value_list",
        val: TokenValue::ValueList,
    },
    KeywordEntry {
        text: "script",
        val: TokenValue::Script,
    },
    KeywordEntry {
        text: "test",
        val: TokenValue::Test,
    },
];

pub fn match_keyword(word: &str) -> Option<TokenValue> {
    KEYWORD_MAP
        .iter()
        .find(|e| e.text == word)
        .map(|entry| entry.val.clone())
}
