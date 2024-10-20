use core::fmt;
use std::{collections::HashMap, io::ErrorKind};

use crate::{burn_script::compiler::BurnScriptCompiler, schema::{AutoEntry, AutoEntryType, Field, LayoutFM, LayoutFMAttribute, Relation, RelationComparison, RelationCriteria, Schema, Script, SerialTrigger, Table, TableOccurrence, Test, Validation, ValidationTrigger, ValidationType, ValueList, ValueListDefinition, ValueListSortBy}};

use super::token::{Token, TokenType};

#[derive(Debug, PartialEq, Eq)]
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
                let mut fmt_string = String::new();

                let val1 = if token.ttype == TokenType::Identifier {
                    format!("{}: \"{}\"", token.ttype.to_string(), token.value)
                } else {
                    format!("\"{}\"", token.ttype.to_string())
                };
                if expected.len() > 1 {
                    write!(f, "Unexpected {} @ {},{}. Expected one of: {:?}", 
                        val1,
                        token.location.line,
                        token.location.column,
                        expected.iter().map(|t| t.to_string()).collect::<Vec<_>>())
                } else {
                    write!(f, "Unexpected {} @ {},{}. Expected: {:?}", 
                        val1,
                        token.location.line,
                        token.location.column,
                        expected.iter().map(|t| t.to_string()).collect::<Vec<_>>())
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
    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?
        .value.parse().expect("Unable to parse object ID.");
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
    expect(tokens, &vec![TokenType::CloseBrace], info)?;

    Ok((id_, script_.get(0).expect("").clone()))
}

pub fn parse_value_list_attributes(tokens: &[Token], info: &mut ParseInfo) -> Result<(Option<String>, ValueListSortBy), ParseErr> {
    let mut from_ = None;
    let mut sort_ = ValueListSortBy::FirstField;

    loop {
        match expect(tokens, &vec![TokenType::From, TokenType::Sort, TokenType::CloseBrace], info)?.ttype {
            TokenType::From => {
                expect(tokens, &vec![TokenType::Assignment], info)?;
                from_ = Some(expect(tokens, &vec![TokenType::Identifier], info)?.value.clone());
                info.cursor += 1;
                if let Some(token) = tokens.get(info.cursor) {
                    match token.ttype {
                        TokenType::Comma => {
                            continue;
                        },
                        TokenType::CloseBrace => {
                            return Ok((from_, sort_))
                        }
                        _ => {
                            return Err(ParseErr::UnexpectedToken { 
                                token: token.clone(),
                                expected: vec![TokenType::Comma, TokenType::CloseBrace] 
                            })
                        }
                    }
                } else {
                    return Err(ParseErr::UnexpectedEOF);
                }
            }
            TokenType::Sort => {
                expect(tokens, &vec![TokenType::Assignment], info)?;
                let order = expect(tokens,
                    &vec![TokenType::FirstField, TokenType::SecondField],
                    info)?;

                sort_ = match order.ttype {
                    TokenType::FirstField => ValueListSortBy::FirstField,
                    TokenType::SecondField => ValueListSortBy::SecondField,
                    _ => unreachable!()
                };
                info.cursor += 1;
                if let Some(token) = tokens.get(info.cursor) {
                    match token.ttype {
                        TokenType::Comma => {
                            continue;
                        },
                        TokenType::CloseBrace => {
                            return Ok((from_, sort_))
                        }
                        _ => {
                            return Err(ParseErr::UnexpectedToken { 
                                token: token.clone(),
                                expected: vec![TokenType::Comma, TokenType::CloseBrace] 
                            })
                        }
                    }
                } else {
                    return Err(ParseErr::UnexpectedEOF);
                }

            }
            TokenType::CloseBrace => {
                return Ok((from_, sort_));
            }
            _ => unreachable!()
        }
    }
}

pub fn parse_value_list(tokens: &[Token], info: &mut ParseInfo) -> Result<(usize, ValueList), ParseErr> {
    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?.value
        .parse::<usize>().expect("Unable to parse object ID.");
    let name_ = expect(tokens, &vec![TokenType::Identifier], info)?.value.clone();
    if expect(tokens, &vec![TokenType::Assignment, TokenType::Colon], info)?.ttype 
        == TokenType::Colon {
            let mut sort_ = ValueListSortBy::FirstField;
            let mut from_: Option<String> = None;
            let first_field = expect(tokens, &vec![TokenType::FieldReference], info)?.value.clone();
            info.cursor += 1;
            let token = &tokens[info.cursor];

            match token.ttype {
                TokenType::Table | TokenType::TableOccurrence | 
                    TokenType::Script | TokenType::ValueList | 
                    TokenType::Relation | TokenType::Test | 
                    TokenType::EOF => {
                        info.cursor -= 1;
                        return Ok((id_, ValueList {
                            id: id_,
                            name: name_,
                            created_by: String::from("admin"),
                            modified_by: String::from("admin"),
                            definition: ValueListDefinition::FromField { 
                                field1: first_field, 
                                field2: None, 
                                from: None, 
                                sort: sort_, }

                        }))
                },
                TokenType::Comma => {
                    let second_field = expect(tokens, &vec![TokenType::FieldReference], info)?.value.clone();
                    info.cursor += 1;
                    let token = &tokens[info.cursor];

                    match token.ttype {
                        TokenType::Table | TokenType::TableOccurrence | 
                            TokenType::Script | TokenType::ValueList | 
                            TokenType::Relation | TokenType::Test | 
                            TokenType::EOF  => {
                                info.cursor -= 1;
                                return Ok((id_, ValueList {
                                    id: id_,
                                    name: name_,
                                    created_by: String::from("admin"),
                                    modified_by: String::from("admin"),
                                    definition: ValueListDefinition::FromField { 
                                        field1: first_field, 
                                        field2: Some(second_field), 
                                        from: from_, 
                                        sort: sort_, }

                                }))
                        },
                        TokenType::Assignment => {
                            expect(tokens, &vec![TokenType::OpenBrace], info)?;
                            (from_, sort_) = parse_value_list_attributes(tokens, info).expect("Unable to parse value list attributes.");
                            return Ok((id_, ValueList {
                                id: id_,
                                name: name_,
                                created_by: String::from("admin"),
                                modified_by: String::from("admin"),
                                definition: ValueListDefinition::FromField { 
                                    field1: first_field, 
                                    field2: Some(second_field), 
                                    from: from_, 
                                    sort: sort_, }

                            }))
                        }
                        _ => {
                            return Err(ParseErr::UnexpectedToken { 
                                token: token.clone(),
                                expected: vec![
                                    TokenType::Table, TokenType::TableOccurrence,
                                    TokenType::Script, TokenType::ValueList,
                                    TokenType::Relation, TokenType::Test,
                                    TokenType::Assignment]  
                            });
                        }
                    }
                }
                TokenType::Assignment => {
                    expect(tokens, &vec![TokenType::OpenBrace], info)?;
                    (from_, sort_) = parse_value_list_attributes(tokens, info).expect("Unable to parse value list attributes.");
                    return Ok((id_, ValueList {
                        id: id_,
                        name: name_,
                        created_by: String::from("admin"),
                        modified_by: String::from("admin"),
                        definition: ValueListDefinition::FromField { 
                            field1: first_field, 
                            field2: None, 
                            from: from_, 
                            sort: sort_, }

                    }))
                }
                _ => {
                    return Err(ParseErr::UnexpectedToken { 
                        token: token.clone(),
                        expected: vec![
                            TokenType::Table, TokenType::TableOccurrence,
                            TokenType::Script, TokenType::ValueList,
                            TokenType::Test, TokenType::Comma, 
                            TokenType::Relation, TokenType::Assignment]  
                    });
                }
            }

    }
    expect(tokens, &vec![TokenType::OpenBrace], info)?;
    info.cursor += 1;

    let mut values = vec![];
    while let Some(token) = tokens.get(info.cursor) {
        match token.ttype {
            TokenType::String => {
                values.push(token.value.clone());
                let token = expect(tokens, &vec![TokenType::Comma, TokenType::CloseBrace], info)?;
                match token.ttype {
                    TokenType::CloseBrace => {
                        break;
                    },
                    TokenType::Comma => {}
                    _ => {
                        return Err(ParseErr::UnexpectedToken { 
                            token: token.clone(),
                            expected: vec![
                                TokenType::CloseBrace,
                                TokenType::Comma,
                            ] 
                        })
                    }
                }

            },
            TokenType::CloseBrace => {
                break;
            },
            _ => {
                return Err(ParseErr::UnexpectedToken { 
                    token: token.clone(), 
                    expected: vec![TokenType::String, TokenType::Comma, TokenType::CloseBrace] 
                })
            }
        }
        info.cursor += 1;
    }

    Ok((id_, ValueList {
        id: id_,
        name: name_,
        created_by: String::from("admin"),
        modified_by: String::from("admin"),
        definition: ValueListDefinition::CustomValues(values)
    }))
}

pub fn parse_test(tokens: &[Token], info: &mut ParseInfo) -> Result<(usize, Test), ParseErr> {
    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?
        .value.parse::<usize>().expect("Unable to parse object id.");
    let name_ = expect(tokens, &vec![TokenType::Identifier], info)?
        .value.clone();
    expect(tokens, &vec![TokenType::Assignment], info)?;
    expect(tokens, &vec![TokenType::OpenBrace], info)?;

    let code = expect(tokens, &vec![TokenType::ScriptContent], info)?;
    let mut script_ = BurnScriptCompiler::compile_burn_script(code.value.as_str());
    script_[0].name = name_.clone();
    expect(tokens, &vec![TokenType::CloseBrace], info)?;

    Ok((id_, Test {
        id: id_,
        name: name_,
        script: script_[0].clone()
    }))
}

pub fn parse_relation_criteria<'a>(tokens: &'a [Token], info: &mut ParseInfo) -> Result<(&'a str, &'a str, RelationComparison), ParseErr> {
    let lhs = expect(tokens, &vec![TokenType::FieldReference], info)?.value.as_str();
    info.cursor += 1;
    let comparison_ = match tokens[info.cursor].ttype {
        TokenType::Eq => RelationComparison::Equal,
        TokenType::Neq => RelationComparison::NotEqual,
        TokenType::Gt => RelationComparison::Greater,
        TokenType::Gte => RelationComparison::GreaterEqual,
        TokenType::Lt => RelationComparison::Less,
        TokenType::Lte => RelationComparison::LessEqual,
        TokenType::Cartesian => RelationComparison::Cartesian,
        _ => return Err(ParseErr::UnexpectedToken { 
            token: tokens[info.cursor].clone(),
            expected: vec![TokenType::Eq, TokenType::Neq, TokenType::Gt,
            TokenType::Gte, TokenType::Lt, TokenType::Lte, TokenType::Cartesian] 
        })
    };
    let rhs = expect(tokens, &vec![TokenType::FieldReference], info)?.value.as_str();
    Ok((lhs, rhs, comparison_ ))
}

pub fn parse_relation(tokens: &[Token], info: &mut ParseInfo) -> Result<(usize, Relation), ParseErr> {
    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?
        .value.parse::<usize>().expect("Unable to parse object id.");
    expect(tokens, &vec![TokenType::Assignment], info)?;
    let token = expect(tokens, &vec![TokenType::OpenBrace, TokenType::FieldReference], info)?;

    let mut criterias_ = vec![];
    let mut tables = [None; 2];
    if token.ttype == TokenType::OpenBrace {
        while let Some(token) = tokens.get(info.cursor) {
            // parse attribute
            let (mut lhs, rhs, comp) = parse_relation_criteria(tokens, info)?;
            let lhs_table = lhs.split("::").collect::<Vec<_>>()[0];
            let rhs_table = rhs.split("::").collect::<Vec<_>>()[0];
            if tables.iter().any(|t| t.is_none()) {
                tables[0] = Some(lhs_table);
                tables[1] = Some(rhs_table);
            }
            if !tables.iter().any(|search| search.unwrap().to_string() == lhs_table) {
                return Err(ParseErr::RelationCriteria { token: tokens[info.cursor - 2].clone() })
            }
            if !tables.iter().any(|search| search.unwrap().to_string() == rhs_table) {
                return Err(ParseErr::RelationCriteria { token: tokens[info.cursor].clone() })
            }

            criterias_.push(RelationCriteria::ByName { field1: lhs.to_string(), field2: rhs.to_string(), comparison: comp});
            let mut token = expect(tokens, &vec![TokenType::Comma, TokenType::CloseBrace], info)?;
            if token.ttype == TokenType::Comma {
                if let Some(end) = tokens.get(info.cursor + 1) {
                    if end.ttype == TokenType::CloseBrace {
                        token = end;
                        info.cursor += 1;
                    }
                }
            }
            if token.ttype == TokenType::CloseBrace {
                return Ok((id_, Relation {
                    id: id_,
                    table1: 0,
                    table1_data_source: 0,
                    table1_name: String::from(tables[0].unwrap()),
                    table2: 0,
                    table2_data_source: 0,
                    table2_name: String::from(tables[1].unwrap()),
                    criterias: criterias_,
                }))
            }
        }
        unreachable!()
    } else {
        info.cursor -= 1;
        let (lhs, rhs, comp) = parse_relation_criteria(tokens, info)?;
        let lhs_table = lhs.split("::").collect::<Vec<_>>()[0];
        let rhs_table = rhs.split("::").collect::<Vec<_>>()[0];
        criterias_.push(RelationCriteria::ByName { field1: lhs.to_string(), field2: rhs.to_string(), comparison: comp });
        return Ok((id_, Relation {
            id: id_,
            table1: 0,
            table1_data_source: 0,
            table1_name: String::from(lhs_table),
            table2: 0,
            table2_data_source: 0,
            table2_name: String::from(rhs_table),
            criterias: criterias_,
        }))
    }
}

pub fn parse_layout_attributes(tokens: &[Token], info: &mut ParseInfo) -> Result<Vec<LayoutFMAttribute>, ParseErr> {
    let mut attributes = vec![];
    while let Some(token) = tokens.get(info.cursor) {
        match token.ttype {
            TokenType::CloseBrace => {
                return Ok(attributes);
            }
            _ => {
                unimplemented!()
            }
        }
        info.cursor += 1;
    }
    unreachable!()
}

pub fn parse_layout(tokens: &[Token], info: &mut ParseInfo) -> Result<(usize, LayoutFM), ParseErr> {
    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?
        .value.parse::<usize>().expect("Unable to parse object id.");
    let name_ = expect(tokens, &vec![TokenType::Identifier], info)?
        .value.clone();


    expect(tokens, &vec![TokenType::Colon], info)?;

    let occurrence = expect(tokens, &vec![TokenType::Identifier], info)?.value.clone();

    expect(tokens, &vec![TokenType::OpenBrace], info)?;

    info.cursor += 1;
    let attrs = parse_layout_attributes(tokens, info)?;

    Ok((1, LayoutFM {
        id: 1,
        name: String::from("Person"),
        table_occurrence: 2,
        table_occurrence_name: occurrence,
    }))
}

pub fn parse(tokens: &Vec<Token>) -> Result<Schema, ParseErr> {
    let mut result = Schema::new();
    let mut info =  ParseInfo { cursor: 0 };
    
    loop {
        match &tokens[info.cursor].ttype {
            TokenType::Table => {
                let (id, table) = parse_table(tokens, &mut info)?;
                result.tables.insert(id, table);
            },
            TokenType::TableOccurrence => {
                let (id, table_occurrence) = parse_table_occurrence(tokens, &mut info)?;
                result.table_occurrences.insert(id, table_occurrence);
            }
            TokenType::Relation => {
                let (id, relation) = parse_relation(tokens, &mut info)?;
                result.relations.insert(id, relation);
            },
            TokenType::ValueList => {
                let (id, valuelist) = parse_value_list(tokens, &mut info)?;
                result.value_lists.insert(id, valuelist);
            },
            TokenType::Script => {
                let (id, script) = parse_script(tokens, &mut info)?;
                result.scripts.insert(id, script);
            },
            TokenType::Layout => {
                let (id, layout) = parse_layout(tokens, &mut info)?;
                result.layouts.insert(id, layout);
            },
            TokenType::Test => {
                let (id, test) = parse_test(tokens, &mut info)?;
                result.tests.insert(id, test);
            },
            TokenType::EOF => {
                break;
            }
            _ => { return Err(ParseErr::UnexpectedToken { 
                token: tokens[info.cursor].clone(), 
                expected: [
                    TokenType::Table, TokenType::TableOccurrence, TokenType::Relation,
                    TokenType::ValueList, TokenType::Script, TokenType::Test,
                    TokenType::EOF,
                ].to_vec(),
            }) }
        }
        if tokens[info.cursor].ttype == TokenType::EOF {
            return Ok(result)
        }
        info.cursor += 1;
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs::read_to_string};

    use crate::{cadlang::{lexer::lex, token::{Location, Token, TokenType}}, schema::{AutoEntry, AutoEntryType, Field, Relation, RelationComparison, RelationCriteria, SerialTrigger, Table, TableOccurrence, Validation, ValidationTrigger, ValidationType, ValueList, ValueListDefinition, ValueListSortBy}};

    use super::{parse, ParseErr};

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
    fn custom_value_list() {
        let code = "
        value_list %1 basic = {
            \"hello\", \"world\", \"this is a test\",
        }";

        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.");

        let expected = ValueList {
            id: 1,
            name: String::from("basic"),
            created_by: String::from("admin"),
            modified_by: String::from("admin"),
            definition: ValueListDefinition::CustomValues(vec![
                String::from("hello"),
                String::from("world"),
                String::from("this is a test")
            ])
        };
        assert!(schema.value_lists.len() == 1);
        assert_eq!(schema.value_lists[&1], expected);
    }

    #[test]
    fn field_value_list() {
        let code = "
        value_list %1 basic : Person_occ::name, Person_occ::id
        ";
        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.");

        let expected = ValueList {
            id: 1,
            name: String::from("basic"),
            created_by: String::from("admin"),
            modified_by: String::from("admin"),
            definition: ValueListDefinition::FromField { 
                field1: "Person_occ::name".to_string(), 
                field2: Some("Person_occ::id".to_string()), 
                from: None, 
                sort: ValueListSortBy::FirstField, 
            }
        };
        assert!(schema.value_lists.len() == 1);
        assert_eq!(schema.value_lists[&1], expected);
    }

    #[test]
    fn field_value_list_single() {
        let code = "
        value_list %1 basic : Person_occ::name
        ";
        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.");

        let expected = ValueList {
            id: 1,
            name: String::from("basic"),
            created_by: String::from("admin"),
            modified_by: String::from("admin"),
            definition: ValueListDefinition::FromField { 
                field1: "Person_occ::name".to_string(), 
                field2: None,
                from: None, 
                sort: ValueListSortBy::FirstField, 
            }
        };
        assert!(schema.value_lists.len() == 1);
        assert_eq!(schema.value_lists[&1], expected);
    }

    #[test]
    fn field_value_list_single_with_options() {
        let code = "
        value_list %1 basic : Person_occ::name = {
            from = Salary_occ,
            sort = second_field
        }
        ";
        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.");

        let expected = ValueList {
            id: 1,
            name: String::from("basic"),
            created_by: String::from("admin"),
            modified_by: String::from("admin"),
            definition: ValueListDefinition::FromField { 
                field1: "Person_occ::name".to_string(), 
                field2: None,
                from: Some(String::from("Salary_occ")), 
                sort: ValueListSortBy::SecondField, 
            }
        };
        assert!(schema.value_lists.len() == 1);
        assert_eq!(schema.value_lists[&1], expected);
    }

    #[test]
    fn field_value_list_double_with_from_option() {
        let code = "
        value_list %1 basic : Person_occ::name = {
            from = Salary_occ,
        }
        ";
        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.");

        let expected = ValueList {
            id: 1,
            name: String::from("basic"),
            created_by: String::from("admin"),
            modified_by: String::from("admin"),
            definition: ValueListDefinition::FromField { 
                field1: "Person_occ::name".to_string(), 
                field2: None,
                from: Some(String::from("Salary_occ")), 
                sort: ValueListSortBy::FirstField, 
            }
        };
        assert!(schema.value_lists.len() == 1);
        assert_eq!(schema.value_lists[&1], expected);
    }

    #[test]
    fn field_value_list_double_with_sort_option() {
        let code = "
        value_list %1 basic : Person_occ::name = {
            sort = second_field,
        }
        ";
        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.");

        let expected = ValueList {
            id: 1,
            name: String::from("basic"),
            created_by: String::from("admin"),
            modified_by: String::from("admin"),
            definition: ValueListDefinition::FromField { 
                field1: "Person_occ::name".to_string(), 
                field2: None,
                from: None,
                sort: ValueListSortBy::SecondField, 
            }
        };
        assert!(schema.value_lists.len() == 1);
        assert_eq!(schema.value_lists[&1], expected);
    }

    #[test]
    fn value_list_single_followed_by_object() {
        let code = "
        value_list %1 basic : Person_occ::name

        relation %2 = Person_occ::salary_id == Salary::id
            ";
        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.");

        let expected_valuelist = ValueList {
            id: 1,
            name: String::from("basic"),
            created_by: String::from("admin"),
            modified_by: String::from("admin"),
            definition: ValueListDefinition::FromField { 
                field1: String::from("Person_occ::name"),
                field2: None,
                from: None,
                sort: ValueListSortBy::FirstField 
            }
        };
        assert_eq!(schema.value_lists[&1], expected_valuelist);
    }

    #[test]
    fn field_value_list_double_with_options() {
        let code = "
        value_list %1 basic : Person_occ::name, Person_occ::id = {
            from = Salary_occ,
            sort = second_field
        }
        ";
        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.");

        let expected = ValueList {
            id: 1,
            name: String::from("basic"),
            created_by: String::from("admin"),
            modified_by: String::from("admin"),
            definition: ValueListDefinition::FromField { 
                field1: "Person_occ::name".to_string(), 
                field2: Some(String::from("Person_occ::id")),
                from: Some(String::from("Salary_occ")), 
                sort: ValueListSortBy::SecondField, 
            }
        };
        assert!(schema.value_lists.len() == 1);
        assert_eq!(schema.value_lists[&1], expected);
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

    #[test]
    fn relation_single_criteria_test() {
        let code = "
        relation %1 = Person_occ::job_id == Job_occ::id
        ";
        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.");
        let expected = Relation {
            id: 1,
            table1_data_source: 0,
            table1_name: String::from("Person_occ"),
            table1: 0,
            table2_data_source: 0,
            table2_name: String::from("Job_occ"),
            table2: 0,
            criterias: vec![RelationCriteria::ByName {
                field1: String::from("job_id"),
                comparison: RelationComparison::Equal,
                field2: String::from("id")
            }]
        };
    }

    #[test]
    fn relation_compound_criteria_test() {
        let code = "
        relation %1 = {
            Person_occ::job_id == Job_occ::id,
            Person_occ::first_name != Job_occ::name
        }
        ";
        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.");
        let expected = Relation {
            id: 1,
            table1_data_source: 0,
            table1_name: String::from("Person_occ"),
            table1: 0,
            table2_data_source: 0,
            table2_name: String::from("Job_occ"),
            table2: 0,
            criterias: vec![
                RelationCriteria::ByName {
                    field1: String::from("job_id"),
                    comparison: RelationComparison::Equal,
                    field2: String::from("id")
                },
                RelationCriteria::ByName {
                    field1: String::from("name"),
                    comparison: RelationComparison::NotEqual,
                    field2: String::from("first_name")
                }
            ]
        };
    }

    #[test]
    fn relation_invalid_criteria_test() {
        let code = "
        relation %1 = {
            Person_occ::job_id == Job_occ::id,
            Person_occ::first_name != Salary_occ::value,
        }
        ";
        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens);
        let expected = ParseErr::RelationCriteria { 
            token: Token::with_value(
                       TokenType::FieldReference, 
                       Location { line: 3, column: 35 }, 
                       String::from("Salary_occ::value"),
                       ) };
        assert!(schema.is_err_and(|e| e ==  expected));
    }

    #[test]
    fn compile_initial_cad() {
        let code = read_to_string("test_data/cad_files/initial.cad").expect("Unable to read file.");

        let tokens = lex(&code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.");
    }
}
