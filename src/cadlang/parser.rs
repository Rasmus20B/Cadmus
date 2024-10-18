use core::fmt;

use crate::schema::{Schema, Table, TableOccurrence};

use super::token::{Token, TokenType};

#[derive(Debug)]
pub enum ParseErr {
    UnexpectedToken { token: Token, expected: Vec<TokenType>},
    RelationCriteria { token: Token }, // criteria must have uniform tables.
    UnknownTable { token: Token },
    UnknownTableOccurrence { token: Token },
    UnknownField { token: Token },
    InvalidAssert { token: Token }, // Asserts can only be used in tests
}

impl<'a> fmt::Display for ParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            _ => write!(f, "nah not compiling.")
        }
    }

}

struct ParseInfo {
    cursor: usize,
}

pub fn parse_table<'a>(tokens: &'a Vec<Token>, info: ParseInfo) -> Result<Table, ParseErr> {
    unimplemented!()
}

pub fn parse_table_occurrence<'a>(tokens: &'a Vec<Token>, info: ParseInfo) -> Result<TableOccurrence, ParseErr> {
    unimplemented!()
}

pub fn parse<'a>(tokens: &'a Vec<Token>) -> Result<Schema, ParseErr> {
    let result = Schema::new();
    let mut info =  ParseInfo { cursor: 0 };
    
    match &tokens[info.cursor].ttype {
        TokenType::Table => {

        },
        TokenType::TableOccurrence => {

        }
        TokenType::Relation => {

        },
        TokenType::ValueList => {

        },
        TokenType::Script => {

        },
        TokenType::Test => {

        },
        token_ => { return Err(ParseErr::UnexpectedToken { token: tokens[info.cursor].clone(), expected: [
            TokenType::Table, TokenType::TableOccurrence, TokenType::Relation,
            TokenType::ValueList, TokenType::Script, TokenType::Test].to_vec(),
        }) }
    };

    Ok(result)
}
