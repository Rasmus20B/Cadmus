use core::fmt;
use std::collections::HashMap;

use crate::schema::{AutoEntry, AutoEntryType, Field, Schema, Table, TableOccurrence, Validation, ValidationTrigger};

use super::token::{Token, TokenType};

#[derive(Debug)]
pub enum ParseErr {
    UnexpectedToken { token: Token, expected: Vec<TokenType>},
    RelationCriteria { token: Token }, // criteria must have uniform tables.
    UnknownTable { token: Token },
    UnknownTableOccurrence { token: Token },
    UnknownField { token: Token },
    InvalidAssert { token: Token }, // Asserts can only be used in tests
    UnexpectedEOF,
}

impl<'a> fmt::Display for ParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            _ => write!(f, "nah not compiling.")
        }
    }

}

pub struct ParseInfo {
    cursor: usize,
}

fn expect<'a>(tokens: &'a Vec<Token>, expected: &Vec<TokenType>, info: &mut ParseInfo) -> Result<&'a Token, ParseErr> {
    info.cursor += 1;
    if let Some(token) = tokens.get(info.cursor) {
        if !expected.contains(&token.ttype) {
            return Err(ParseErr::UnexpectedToken { 
                token: token.clone(), 
                expected: expected.to_vec(),
            })
        } else {
            return Ok(&token)
        }
    } else {
        return Err(ParseErr::UnexpectedEOF)
    }
}

pub fn parse_field<'a>(tokens: &'a Vec<Token>, mut info: &mut ParseInfo) -> Result<(usize, Field), ParseErr> {
    println!("DOES GET HERE");
    let tmp = Field {
        id: 0,
        name: String::from("field"),
        created_by: String::new(),
        modified_by: String::new(),
        autoentry: AutoEntry {
            definition: AutoEntryType::NA,
            nomodify: false,
        },
        validation: Validation {
            trigger: ValidationTrigger::OnEntry,
            user_override: false,
            checks: vec![],
            message: String::from("Error with validation."),
        },
        global: false,
        repetitions: 1,
    };

    expect(tokens, &vec![TokenType::ObjectNumber], &mut info)?;
    expect(tokens, &vec![TokenType::Identifier], &mut info)?;
    expect(tokens, &vec![TokenType::Assignment], &mut info)?;
    expect(tokens, &vec![TokenType::OpenBrace], &mut info)?;
    info.cursor += 1;

    let mut depth = 1;
    while let Some(token) = tokens.get(info.cursor) {
        match token.ttype {
            TokenType::CloseBrace => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                info.cursor += 1;
            },
            TokenType::OpenBrace => {
                depth += 1;
                info.cursor += 1;
            }
            _ => {
                info.cursor += 1;
            }
        }
    }

    if tokens.get(info.cursor).is_none() {
        return Err(ParseErr::UnexpectedEOF);
    }
    println!("Scanned a field.");
    Ok((0, tmp))
}

pub fn parse_table<'a>(tokens: &'a Vec<Token>, mut info: &mut ParseInfo) -> Result<(usize, Table), ParseErr> {
    info.cursor += 1;
    let id_ = tokens.get(info.cursor).expect("Unexpected end of file.").value.parse().expect("Unable to parse object ID.");
    info.cursor += 1;
    let name_ = tokens.get(info.cursor).expect("Unexpected end of file.").value.clone();
    expect(tokens, &vec![TokenType::Assignment], &mut info)?;
    expect(tokens, &vec![TokenType::OpenBrace], &mut info)?;
    info.cursor += 1;
    let mut fields_ = HashMap::new();
    while let Some(token) = tokens.get(info.cursor) {
        println!("CURSOR: {}, Token: {:?}", info.cursor, token.ttype);
        match token.ttype {
            TokenType::Field => {
                let tmp = parse_field(tokens, &mut info)?;
                fields_.insert(tmp.0, tmp.1);
                println!("Token after field: {:?} \n{:?}", tokens[info.cursor - 1], tokens[info.cursor]);
            },
            TokenType::CloseBrace => {
                break;
            }
            TokenType::Comma => {
                println!("FOUND A COMMA.");
            }
            _ => return Err(ParseErr::UnexpectedToken { 
                token: token.clone(), 
                expected: vec![TokenType::Field, TokenType::CloseBrace] 
            })
        }
        info.cursor += 1;
    }
    if tokens.get(info.cursor).is_none() {
        return Err(ParseErr::UnexpectedEOF);
    }

    println!("FINIHED TABLE DEFINITION.");
    Ok((id_, Table {
        id: id_,
        name: name_,
        created_by: String::from("admin"),
        modified_by: String::from("admin"),
        fields: fields_,
    }))
}

pub fn parse_table_occurrence<'a>(tokens: &'a Vec<Token>, mut info: &mut ParseInfo) -> Result<(usize, TableOccurrence), ParseErr> {

    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], &mut info)?.value.parse::<usize>()
        .expect("Unable to parse object id.");

    println!("ID: {}", id_);
    let name_ = expect(tokens, &vec![TokenType::Identifier], &mut info)?.value.clone();
    expect(tokens, &vec![TokenType::Colon], &mut info)?;
    let base_table_ = expect(tokens, &vec![TokenType::Identifier], &mut info)?.value.clone();

    Ok((id_, TableOccurrence {
        id: id_,
        created_by: String::from("admin"),
        modified_by: String::from("admin"),
        name: name_,
        table_actual: 0,
        table_actual_name: base_table_,
    }))
}

pub fn parse<'a>(tokens: &'a Vec<Token>) -> Result<Schema, ParseErr> {
    let mut result = Schema::new();
    let mut info =  ParseInfo { cursor: 0 };
    
    loop {
        match &tokens[info.cursor].ttype {
            TokenType::Table => {
                let (id, table) = parse_table(tokens, &mut info)
                    .expect("Unable to parse table.");
                result.tables.insert(id, table);
            },
            TokenType::TableOccurrence => {
                let (id, table_occurrence) = parse_table_occurrence(tokens, &mut info)
                    .expect("unable to parse table occurrence.");
                println!("id: {:?}", table_occurrence);
                result.table_occurrences.insert(id, table_occurrence);
            }
            TokenType::Relation => {
            },
            TokenType::ValueList => {
            },
            TokenType::Script => {
            },
            TokenType::Test => {
            },
            TokenType::EOF => {
                break;
            }
            token_ => { return Err(ParseErr::UnexpectedToken { token: tokens[info.cursor].clone(), expected: [
                TokenType::Table, TokenType::TableOccurrence, TokenType::Relation,
                TokenType::ValueList, TokenType::Script, TokenType::Test].to_vec(),
            }) }
        }
        info.cursor += 1;
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{cadlang::lexer::lex, schema::{Table, TableOccurrence}};

    use super::parse;

    #[test]
    fn basic_table_parse_test() {
        let code = "
            table %1 Person = {
                field %1 id = {
                    datatype = Number,
                    auto_increment=true,
                    required =true,
                    unique= true,
                    auto_entry = [get(uuid)],
                    validation_message = \"Invalid ID chosen.\",
                }
            }
            ";

        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens);
        let expected = Table::new(1);

    }

    #[test]
    fn basic_table_occurrence_parse_test() {
        let code = "
            table %1 Person = {
                field %1 id = {
                    datatype = Number,
                    auto_increment=true,
                    required =true,
                    unique= true,
                    auto_entry = [get(uuid)],
                    validation_message = \"Invalid ID chosen.\",
                }
            }
            table_occurrence %1 Person_occ : Person
            ";

        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.");

        let expected = TableOccurrence {
            id: 1,
            created_by: String::from("admin"),
            modified_by: String::from("admin"),
            name: String::from("Person_occ"),
            table_actual: 0,
            table_actual_name: String::from("Person")
        };
        assert_eq!(expected, schema.table_occurrences[&1]);
    }
}
