use core::fmt;
use std::collections::HashMap;

use crate::{burn_script::compiler::BurnScriptCompiler, schema::{AutoEntry, AutoEntryType, Field, Schema, Script, SerialTrigger, Table, TableOccurrence, Validation, ValidationTrigger, ValidationType}};

use super::token::{Token, TokenType};

#[derive(Debug)]
pub enum ParseErr {
    UnexpectedToken { token: Token, expected: Vec<TokenType>},
    RelationCriteria { token: Token }, // criteria must have uniform tables.
    UnknownTable { token: Token },
    UnknownTableOccurrence { token: Token },
    UnknownField { token: Token },
    InvalidAssert { token: Token }, // Asserts can only be used in tests
    MissingAttribute { base_object: String, construct: String, specifier: String },
    UnexpectedEOF,
}

impl<'a> fmt::Display for ParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken { token, expected } => {
                if expected.len() > 1 {
                    write!(f, "Unexpected Token: {:?}.\nExpected one of: {:?}", token, expected)
                } else {
                    write!(f, "Unexpected Token: {:?}.\nExpected {:?}", token, expected)
                }
            }
            Self::RelationCriteria { token } => {
                write!(f, "Found non-matching table \"{}\" reference in relation criteria. @ {}, {}", 
                    token.value,
                    token.location.line,
                    token.location.column)
            }
            Self::UnknownTable { token } => {
                write!(f, "Invalid reference to table: {} @ {}, {}", 
                    token.value,
                    token.location.line,
                    token.location.column)
            }
            Self::UnknownTableOccurrence { token } => {
                write!(f, "Invalid reference to table occurrence: {} @ {}, {}", 
                    token.value,
                    token.location.line,
                    token.location.column)
            }
            Self::UnknownField { token } => {
                write!(f, "Invalid reference to field: {} @ {}, {}", 
                    token.value,
                    token.location.line,
                    token.location.column)
            }
            Self::MissingAttribute { base_object, construct, specifier } => {
                write!(f, "Missing attribute {} for {} in {}", specifier, construct, base_object)
            }
            _ => write!(f, "nah not compiling.")
        }
    }

}

pub struct ParseInfo {
    cursor: usize,
}

fn expect<'a>(tokens: &'a [Token], expected: &Vec<TokenType>, info: &mut ParseInfo) -> Result<&'a Token, ParseErr> {
    info.cursor += 1;
    if let Some(token) = tokens.get(info.cursor) {
        if !expected.contains(&token.ttype) {
            Err(ParseErr::UnexpectedToken { 
                token: token.clone(), 
                expected: expected.to_vec(),
            })
        } else {
            Ok(token)
        }
    } else {
        Err(ParseErr::UnexpectedEOF)
    }
}

pub fn parse_field(tokens: &[Token], info: &mut ParseInfo) -> Result<(usize, Field), ParseErr> {
    let mut tmp = Field {
        id: 0,
        name: String::new(),
        created_by: String::from("admin"),
        modified_by: String::from("admin"),
        autoentry: AutoEntry {
            definition: AutoEntryType::NA,
            nomodify: false,
        },
        validation: Validation {
            trigger: ValidationTrigger::OnEntry,
            user_override: true,
            checks: vec![],
            message: String::from("Error with field validation."),
        },
        global: false,
        repetitions: 1,
    };

    tmp.id = expect(tokens, &vec![TokenType::ObjectNumber], info)?
        .value.parse().expect("Unable to parse object id.");
    tmp.name = expect(tokens, &vec![TokenType::Identifier], info)?
        .value.clone();
    expect(tokens, &vec![TokenType::Assignment], info)?;
    expect(tokens, &vec![TokenType::OpenBrace], info)?;
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

            // Auto Entry Switches
            TokenType::CalculatedVal => {
                expect(tokens, &vec![TokenType::Assignment], info)?;
                let calc = expect(tokens, &vec![TokenType::Exclamation, TokenType::Calculation], info)?;
                match calc.ttype {
                    TokenType::Exclamation => {
                        tmp.autoentry.definition = AutoEntryType::Calculation { 
                            code: expect(tokens, &vec![TokenType::Calculation], info)?
                                .value.clone(),
                            noreplace: true 
                        }

                    },
                    TokenType::Calculation => {
                        tmp.autoentry.definition = AutoEntryType::Calculation { 
                            code: calc.value.clone(),
                            noreplace: false 
                        }
                    },
                    _ => unreachable!()

                }
            }

            TokenType::Serial => {
                expect(tokens, &vec![TokenType::Assignment], info)?;
                expect(tokens, &vec![TokenType::OpenBrace], info)?;
                info.cursor += 1;

                let mut generate_: Option::<SerialTrigger> = None;
                let mut next_: Option<usize> = None;
                let mut increment_: Option<usize> = None;
                while let Some(token) = tokens.get(info.cursor) {
                    match token.ttype {
                        TokenType::Generate => {
                            expect(tokens, &vec![TokenType::Assignment], info)?;
                            generate_ = match expect(tokens, 
                                &vec![TokenType::OnCreation, TokenType::OnCommit], 
                                info)?.ttype {
                                TokenType::OnCreation => {
                                    Some(SerialTrigger::OnCreation)
                                }
                                TokenType::OnCommit => {
                                    Some(SerialTrigger::OnCommit)
                                }
                                _ => unreachable!()
                            };
                        }
                        TokenType::Next => {
                            expect(tokens, &vec![TokenType::Assignment], info)?;
                            next_ = Some(expect(tokens, &vec![TokenType::IntegerLiteral], info)?
                                .value.parse::<usize>().expect("unable to parse next."));
                        }
                        TokenType::Increment => {
                            expect(tokens, &vec![TokenType::Assignment], info)?;
                            increment_ = Some(expect(tokens, &vec![TokenType::IntegerLiteral], info)?
                                .value.parse::<usize>().expect("unable to parse increment."));
                        }
                        TokenType::Comma => {}
                        TokenType::CloseBrace => {
                            info.cursor += 1;
                            break;
                        }
                        _ => {
                            return Err(ParseErr::UnexpectedToken { 
                                token: token.clone(), 
                                expected: vec![TokenType::Generate, TokenType::Next, TokenType::Increment] 
                            });
                        }
                    }
                    info.cursor += 1;
                }

                if generate_.is_none() {
                    return Err(ParseErr::MissingAttribute { 
                        base_object: tmp.name,
                        construct: String::from("Serial"), 
                        specifier: String::from("generate") 
                    })
                }
                if next_.is_none() {
                    return Err(ParseErr::MissingAttribute { 
                        base_object: tmp.name,
                        construct: String::from("Serial"), 
                        specifier: String::from("next") 
                    })
                }
                if increment_.is_none() {
                    return Err(ParseErr::MissingAttribute { 
                        base_object: tmp.name,
                        construct: String::from("Serial"), 
                        specifier: String::from("increment") 
                    })
                }
                tmp.autoentry.definition = AutoEntryType::Serial { 
                    next: next_.unwrap(), 
                    increment: increment_.unwrap(), 
                    trigger: generate_.unwrap() 
                }

            }

            // Validation Switches
            TokenType::NotEmpty => {
                expect(tokens, &vec![TokenType::Assignment], info)?;
                match expect(tokens, &vec![TokenType::True, TokenType::False], info)?.ttype {
                    TokenType::True => {
                        tmp.validation.checks.push(ValidationType::NotEmpty)
                    },
                    TokenType::False => {},
                    _ => unreachable!()
                };
            }
            TokenType::Required => {
                expect(tokens, &vec![TokenType::Assignment], info)?;
                match expect(tokens, &vec![TokenType::True, TokenType::False], info)?.ttype {
                    TokenType::True => {
                        tmp.validation.checks.push(ValidationType::Required)
                    },
                    TokenType::False => {},
                    _ => unreachable!()
                };
            }
            TokenType::Unique => {
                expect(tokens, &vec![TokenType::Assignment], info)?;
                match expect(tokens, &vec![TokenType::True, TokenType::False], info)?.ttype {
                    TokenType::True => {
                        tmp.validation.checks.push(ValidationType::Unique)
                    },
                    TokenType::False => {},
                    _ => unreachable!()
                };
            }
            _ => {
                info.cursor += 1;
            }
        }
    }

    if tokens.get(info.cursor).is_none() {
        return Err(ParseErr::UnexpectedEOF);
    }
    Ok((tmp.id, tmp))
}

pub fn parse_table(tokens: &[Token], info: &mut ParseInfo) -> Result<(usize, Table), ParseErr> {
    info.cursor += 1;
    let id_ = tokens.get(info.cursor).expect("Unexpected end of file.").value.parse().expect("Unable to parse object ID.");
    info.cursor += 1;
    let name_ = tokens.get(info.cursor).expect("Unexpected end of file.").value.clone();
    expect(tokens, &vec![TokenType::Assignment], info)?;
    expect(tokens, &vec![TokenType::OpenBrace], info)?;
    info.cursor += 1;
    let mut fields_ = HashMap::new();
    while let Some(token) = tokens.get(info.cursor) {
        match token.ttype {
            TokenType::Field => {
                let tmp = parse_field(tokens, info)?;
                fields_.insert(tmp.0, tmp.1);
            },
            TokenType::CloseBrace => {
                break;
            }
            TokenType::Comma => {
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
    Ok((id_, Table {
        id: id_,
        name: name_,
        created_by: String::from("admin"),
        modified_by: String::from("admin"),
        fields: fields_,
    }))
}

pub fn parse_table_occurrence(tokens: &[Token], info: &mut ParseInfo) -> Result<(usize, TableOccurrence), ParseErr> {

    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?.value.parse::<usize>()
        .expect("Unable to parse object id.");

    let name_ = expect(tokens, &vec![TokenType::Identifier], info)?.value.clone();
    expect(tokens, &vec![TokenType::Colon], info)?;
    let base_table_ = expect(tokens, &vec![TokenType::Identifier], info)?.value.clone();

    Ok((id_, TableOccurrence {
        id: id_,
        created_by: String::from("admin"),
        modified_by: String::from("admin"),
        name: name_,
        table_actual: 0,
        table_actual_name: base_table_,
    }))
}

pub fn parse_script(tokens: &[Token], info: &mut ParseInfo) -> Result<(usize, Script), ParseErr> {
    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?
        .value.parse::<usize>().expect("Unable to parse object number.");
    let name_ = expect(tokens, &vec![TokenType::Identifier], info)?.value.clone();
    expect(tokens, &vec![TokenType::Assignment], info)?;
    expect(tokens, &vec![TokenType::OpenBrace], info)?;

    let code = expect(tokens, &vec![TokenType::ScriptContent], info)?;
    let mut script_ = BurnScriptCompiler::compile_burn_script(code.value.as_str());
    script_[0].name = name_.clone();

    Ok((id_, script_.get(0).expect("").clone()))
}

pub fn parse(tokens: &Vec<Token>) -> Result<Schema, ParseErr> {
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
                result.table_occurrences.insert(id, table_occurrence);
            }
            TokenType::Relation => {
            },
            TokenType::ValueList => {
            },
            TokenType::Script => {
                let (id, script) = parse_script(tokens, &mut info)
                    .expect("Unable to parse script.");
                result.scripts.insert(id, script);
            },
            TokenType::Test => {
            },
            TokenType::EOF => {
                break;
            }
            _ => { return Err(ParseErr::UnexpectedToken { 
                token: tokens[info.cursor].clone(), 
                expected: [
                    TokenType::Table, TokenType::TableOccurrence, TokenType::Relation,
                    TokenType::ValueList, TokenType::Script, TokenType::Test
                ].to_vec(),
            }) }
        }
        info.cursor += 1;
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{cadlang::lexer::lex, schema::{AutoEntry, AutoEntryType, Field, SerialTrigger, Table, TableOccurrence, Validation, ValidationTrigger, ValidationType}};

    use super::parse;

    #[test]
    fn basic_table_parse_test() {
        let code = "
            table %1 Person = {
                field %1 id = {
                    datatype = Number,
                    required =true,
                    unique= true,
                    calculated_val = [get(uuid)],
                    validation_message = \"Invalid ID chosen.\",
                },
                field %2 counter = {
                    datatype = Number,
                    serial = {
                        generate = on_creation,
                        next = 1,
                        increment = 1,
                    }
                }
            }
            ";

        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.");
        let mut expected_fields = HashMap::new();
        expected_fields.insert(1, Field {
                    id: 1,
                    name: String::from("id"),
                    created_by: String::from("admin"),
                    modified_by: String::from("admin"),
                    repetitions: 1,
                    global: false,
                    autoentry: AutoEntry {
                        nomodify: false,
                        definition: AutoEntryType::Calculation { 
                            code: String::from("get(uuid)"), 
                            noreplace: false, 
                        }
                    },
                    validation: Validation {
                        checks: vec![
                            ValidationType::Required,
                            ValidationType::Unique
                        ],
                        message: String::from("Error with field validation."),
                        trigger: ValidationTrigger::OnEntry,
                        user_override: true,
                    }
                });
        expected_fields.insert(2, Field {
                    id: 2,
                    name: String::from("counter"),
                    created_by: String::from("admin"),
                    modified_by: String::from("admin"),
                    repetitions: 1,
                    global: false,
                    autoentry: AutoEntry {
                        nomodify: false,
                        definition: AutoEntryType::Serial { 
                            next: 1, 
                            increment: 1, 
                            trigger: SerialTrigger::OnCreation 
                        }
                    },
                    validation: Validation {
                        checks: vec![
                        ],
                        message: String::from("Error with field validation."),
                        trigger: ValidationTrigger::OnEntry,
                        user_override: true,
                    }
                });
        let expected = Table {
            id: 1,
            name: String::from("Person"),
            fields: expected_fields,
            created_by: String::from("admin"),
            modified_by: String::from("admin"),
        };
        assert_eq!(schema.tables[&1], expected);
    }

    #[test]
    fn basic_table_occurrence_parse_test() {
        let code = "
            table %1 Person = {
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