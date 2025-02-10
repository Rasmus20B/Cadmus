use core::fmt;
use std::collections::BTreeMap;

use crate::{burn_script::compiler::BurnScriptCompiler, schema::{DataType, DBObjectReference, LayoutFMAttribute, RelationComparison, Script, SerialTrigger, TableOccurrence, Test, ValidationTrigger}};

use crate::dbobjects::data_source::*;
use super::{staging::*, error::CompileErr};
use super::token::{Token, TokenType};


#[derive(PartialEq, Eq, Debug)]
pub enum FMObjType {
    Table,
    TableOccurrence,
    Field,
    ValueList,
}

impl<'a> fmt::Display for FMObjType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Table => write!(f, "table"),
            Self::TableOccurrence => write!(f, "table occurrence"),
            Self::Field => write!(f, "field"),
            Self::ValueList => write!(f, "valuelist")
        }
    }
}


pub struct ParseInfo {
    cursor: usize,
}

fn expect<'a>(tokens: &'a [Token], expected: &Vec<TokenType>, info: &mut ParseInfo) -> Result<&'a Token, CompileErr> {
    info.cursor += 1;
    if let Some(token) = tokens.get(info.cursor) {
        if !expected.contains(&token.ttype) {
            Err(CompileErr::UnexpectedToken { 
                token: token.clone(), 
                expected: expected.to_vec(),
            })
        } else {
            Ok(token)
        }
    } else {
        Err(CompileErr::UnexpectedEOF)
    }
}

pub fn parse_field(tokens: &[Token], info: &mut ParseInfo) -> Result<(u16, StagedField), CompileErr> {
    let mut checks_ = vec![];
    let validation_msg = String::from("Error with field validation.");
    let validation_trigger = ValidationTrigger::OnEntry;
    let user_override_ = true;
    let mut autoentry_ = StagedAutoEntry {
        definition: StagedAutoEntryType::NA,
        nomodify: false,
    };
    let repetitions_ = 1;
    let global_ = false;
    let mut dtype_ = DataType::Text;

    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?
        .value.parse().expect("Unable to parse object id.");
    let name_ = expect(tokens, &vec![TokenType::Identifier], info)?;

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
                        autoentry_.definition = StagedAutoEntryType::Calculation { 
                            code: expect(tokens, &vec![TokenType::Calculation], info)?
                                .value.clone(),
                            noreplace: true 
                        }

                    },
                    TokenType::Calculation => {
                        autoentry_.definition = StagedAutoEntryType::Calculation { 
                            code: calc.value.clone(),
                            noreplace: false 
                        }
                    },
                    _ => unreachable!()

                }
            }

            TokenType::Datatype => {
                expect(tokens, &vec![TokenType::Assignment], info)?;
                dtype_ = match expect(tokens, &vec![TokenType::Text, TokenType::Number, TokenType::Date], info)?.ttype {
                    TokenType::Text => DataType::Text,
                    TokenType::Number => DataType::Number,
                    TokenType::Date => DataType::Date,
                    _ => todo!("add more datatypes")
                };
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
                            return Err(CompileErr::UnexpectedToken { 
                                token: token.clone(), 
                                expected: vec![TokenType::Generate, TokenType::Next, TokenType::Increment] 
                            });
                        }
                    }
                    info.cursor += 1;
                }

                if generate_.is_none() {
                    return Err(CompileErr::MissingAttribute { 
                        base_object: name_.value.clone(),
                        construct: String::from("Serial"), 
                        specifier: String::from("generate") 
                    })
                }
                if next_.is_none() {
                    return Err(CompileErr::MissingAttribute { 
                        base_object: name_.value.clone(),
                        construct: String::from("Serial"), 
                        specifier: String::from("next") 
                    })
                }
                if increment_.is_none() {
                    return Err(CompileErr::MissingAttribute { 
                        base_object: name_.value.clone(),
                        construct: String::from("Serial"), 
                        specifier: String::from("increment") 
                    })
                }
                autoentry_.definition = StagedAutoEntryType::Serial { 
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
                        checks_.push(StagedValidationType::NotEmpty)
                    },
                    TokenType::False => {},
                    _ => unreachable!()
                };
            }
            TokenType::Required => {
                expect(tokens, &vec![TokenType::Assignment], info)?;
                match expect(tokens, &vec![TokenType::True, TokenType::False], info)?.ttype {
                    TokenType::True => {
                        checks_.push(StagedValidationType::Required)
                    },
                    TokenType::False => {},
                    _ => unreachable!()
                };
            }
            TokenType::Unique => {
                expect(tokens, &vec![TokenType::Assignment], info)?;
                match expect(tokens, &vec![TokenType::True, TokenType::False], info)?.ttype {
                    TokenType::True => {
                        checks_.push(StagedValidationType::Unique)
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
        return Err(CompileErr::UnexpectedEOF);
    }
    Ok((id_, StagedField {
        id: id_,
        name: name_.clone(),
        dtype: dtype_,
        validation: StagedValidation {
            checks: checks_,
            message: validation_msg,
            trigger: validation_trigger,
            user_override: user_override_
        },
        autoentry: autoentry_,
        global: global_,
        repetitions: repetitions_,
    }))
}

pub fn parse_table(tokens: &[Token], info: &mut ParseInfo) -> Result<(u16, StagedTable), CompileErr> {
    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?
        .value.parse::<u16>().expect("Unable to parse object ID.");
    let name_ = expect(tokens, &vec![TokenType::Identifier], info)?;

    expect(tokens, &vec![TokenType::Assignment], info)?;
    expect(tokens, &vec![TokenType::OpenBrace], info)?;
    info.cursor += 1;
    let mut fields_ = BTreeMap::new();
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
            _ => return Err(CompileErr::UnexpectedToken { 
                token: token.clone(), 
                expected: vec![TokenType::Field, TokenType::CloseBrace] 
            })
        }
        info.cursor += 1;
    }
    if tokens.get(info.cursor).is_none() {
        return Err(CompileErr::UnexpectedEOF);
    }
    Ok((id_, StagedTable {
        id: id_,
        name: name_.clone(),
        fields: fields_,
    }))
}

pub fn parse_extern(tokens: &[Token], info: &mut ParseInfo) -> Result<(u16, DataSource), CompileErr> {
    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?.value.parse::<u16>()
        .expect("Unable to parse extern object id.");

    let name_ = expect(tokens, &vec![TokenType::Identifier], info)?;
    expect(tokens, &vec![TokenType::Colon], info)?;

    let filename_ = expect(tokens, &vec![TokenType::String], info)?;

    if filename_.value.ends_with(".cad") {
        return Ok((id_, DataSource {
            id: id_ as u32,
            name: name_.value.clone(),
            paths: vec![filename_.value.clone()],
            dstype: DataSourceType::Cadmus,
        }))
    } else if filename_.value.ends_with(".fmp12") {
        return Ok((id_, DataSource {
            id: id_ as u32,
            name: name_.value.clone(),
            paths: vec![filename_.value.clone()],
            dstype: DataSourceType::FileMaker,
        }))
    } else {
        return Err(CompileErr::UnknownFileType { filename: name_.clone() })
    }

}

pub fn parse_table_occurrence(tokens: &[Token], info: &mut ParseInfo) -> Result<(u16, StagedOccurrence), CompileErr> {

    let result = TableOccurrence {
        id: 0,
        created_by: String::from("admin"),
        modified_by: String::from("admin"),
        name: String::new(),
        base_table: DBObjectReference { data_source: 0, top_id: 0, inner_id: 0 },
    };

    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?.value.parse::<u16>()
        .expect("Unable to parse object id.");

    let name_ = expect(tokens, &vec![TokenType::Identifier], info)?;

    expect(tokens, &vec![TokenType::Colon], info)?;
    let mut table_or_source = expect(tokens, &vec![TokenType::Identifier], info)?;


    if expect(tokens, &vec![TokenType::ScopeResolution], info).is_ok() {
        let table = expect(tokens, &vec![TokenType::Identifier], info)?;
        return Ok((id_,
                StagedOccurrence {
                    id: id_ as usize,
                    name: name_.clone(),
                    data_source: Some(table_or_source.clone()),
                    base_table: table.clone(),
                }))
    } else {
        info.cursor -= 1;
        let table = table_or_source.clone(); 
        Ok((id_, StagedOccurrence {
            id: id_ as usize,
            name: name_.clone(),
            data_source: None,
            base_table: table.clone(),
        }))
    }

}

pub fn parse_script(tokens: &[Token], info: &mut ParseInfo) -> Result<(u16, Script), CompileErr> {
    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?
        .value.parse::<u16>().expect("Unable to parse object number.");
    let name_ = expect(tokens, &vec![TokenType::Identifier], info)?.value.clone();

    expect(tokens, &vec![TokenType::Assignment], info)?;
    expect(tokens, &vec![TokenType::OpenBrace], info)?;

    let code = expect(tokens, &vec![TokenType::ScriptContent], info)?;
    let mut script_ = BurnScriptCompiler::compile_burn_script(code.value.as_str());
    script_[0].name = name_.clone();
    expect(tokens, &vec![TokenType::CloseBrace], info)?;

    Ok((id_, script_.first().expect("").clone()))
}

pub fn parse_value_list_attributes(tokens: &[Token], info: &mut ParseInfo) -> Result<(Option<Token>, StagedValueListSortBy), CompileErr> {
    let mut from_ = None;
    let mut sort_ = StagedValueListSortBy::FirstField;

    loop {
        match expect(tokens, &vec![TokenType::From, TokenType::Sort, TokenType::CloseBrace], info)?.ttype {
            TokenType::From => {
                expect(tokens, &vec![TokenType::Assignment], info)?;
                from_ = Some(expect(tokens, &vec![TokenType::Identifier], info)?);
                info.cursor += 1;
                if let Some(token) = tokens.get(info.cursor) {
                    match token.ttype {
                        TokenType::Comma => {
                            continue;
                        },
                        TokenType::CloseBrace => {
                            return Ok((from_.cloned(), sort_))
                        }
                        _ => {
                            return Err(CompileErr::UnexpectedToken { 
                                token: token.clone(),
                                expected: vec![TokenType::Comma, TokenType::CloseBrace] 
                            })
                        }
                    }
                } else {
                    return Err(CompileErr::UnexpectedEOF);
                }
            }
            TokenType::Sort => {
                expect(tokens, &vec![TokenType::Assignment], info)?;
                let order = expect(tokens,
                    &vec![TokenType::FirstField, TokenType::SecondField],
                    info)?;

                sort_ = match order.ttype {
                    TokenType::FirstField => StagedValueListSortBy::FirstField,
                    TokenType::SecondField => StagedValueListSortBy::SecondField,
                    _ => unreachable!()
                };
                info.cursor += 1;
                if let Some(token) = tokens.get(info.cursor) {
                    match token.ttype {
                        TokenType::Comma => {
                            continue;
                        },
                        TokenType::CloseBrace => {
                            return Ok((from_.cloned(), sort_))
                        }
                        _ => {
                            return Err(CompileErr::UnexpectedToken { 
                                token: token.clone(),
                                expected: vec![TokenType::Comma, TokenType::CloseBrace] 
                            })
                        }
                    }
                } else {
                    return Err(CompileErr::UnexpectedEOF);
                }

            }
            TokenType::CloseBrace => {
                return Ok((from_.cloned(), sort_));
            }
            _ => unreachable!()
        }
    }
}

pub fn parse_value_list(tokens: &[Token], info: &mut ParseInfo) -> Result<(u16, StagedValueList), CompileErr> {
    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?.value
        .parse::<u16>().expect("Unable to parse object ID.");
    let name_ = expect(tokens, &vec![TokenType::Identifier], info)?;

    if expect(tokens, &vec![TokenType::Assignment, TokenType::Colon], info)?.ttype 
        == TokenType::Colon {
            let mut sort_ = StagedValueListSortBy::FirstField;
            let mut from_: Option<Token> = None;
            let occurrence_ = expect(tokens, &vec![TokenType::Identifier], info)?;
            expect(tokens, &vec![TokenType::ScopeResolution], info)?;
            let first_field = expect(tokens, &vec![TokenType::Identifier], info)?;
            info.cursor += 1;
            let token = &tokens[info.cursor];

            match token.ttype {
                TokenType::Table | TokenType::TableOccurrence | 
                    TokenType::Script | TokenType::ValueList | 
                    TokenType::Relation | TokenType::Test | 
                    TokenType::EOF => {
                        info.cursor -= 1;
                        return Ok((id_, StagedValueList {
                            id: id_,
                            name: name_.clone(),
                            definition: StagedValueListDefinition::FromField { 
                                occurrence: occurrence_.clone(),
                                field1: first_field.clone(), 
                                field2: None, 
                                from: None, 
                                sort: sort_, }

                        }))
                },
                TokenType::Comma => {
                    let second_table = expect(tokens, &vec![TokenType::Identifier], info)?;
                    expect(tokens, &vec![TokenType::ScopeResolution], info)?;
                    let second_field = expect(tokens, &vec![TokenType::Identifier], info)?;
                    info.cursor += 1;
                    let token = &tokens[info.cursor];

                    match token.ttype {
                        TokenType::Table | TokenType::TableOccurrence | 
                            TokenType::Script | TokenType::ValueList | 
                            TokenType::Relation | TokenType::Test | 
                            TokenType::EOF  => {
                                info.cursor -= 1;
                                return Ok((id_, StagedValueList {
                                    id: id_,
                                    name: name_.clone(),
                                    definition: StagedValueListDefinition::FromField { 
                                        occurrence: occurrence_.clone(),
                                        field1: first_field.clone(), 
                                        field2: Some(second_field.clone()), 
                                        from: from_, 
                                        sort: sort_, }

                                }))
                        },
                        TokenType::Assignment => {
                            expect(tokens, &vec![TokenType::OpenBrace], info)?;
                            (from_, sort_) = parse_value_list_attributes(tokens, info).expect("Unable to parse value list attributes.");
                            return Ok((id_, StagedValueList {
                                id: id_,
                                name: name_.clone(),
                                definition: StagedValueListDefinition::FromField { 
                                    occurrence: occurrence_.clone(),
                                    field1: first_field.clone(), 
                                    field2: Some(second_field.clone()), 
                                    from: from_, 
                                    sort: sort_, }

                            }))
                        }
                        _ => {
                            return Err(CompileErr::UnexpectedToken { 
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
                    return Ok((id_, StagedValueList {
                        id: id_,
                        name: name_.clone(),
                        definition: StagedValueListDefinition::FromField { 
                            occurrence: occurrence_.clone(),
                            field1: first_field.clone(), 
                            field2: None, 
                            from: from_, 
                            sort: sort_, }

                    }))
                }
                _ => {
                    return Err(CompileErr::UnexpectedToken { 
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
                        return Err(CompileErr::UnexpectedToken { 
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
                return Err(CompileErr::UnexpectedToken { 
                    token: token.clone(), 
                    expected: vec![TokenType::String, TokenType::Comma, TokenType::CloseBrace] 
                })
            }
        }
        info.cursor += 1;
    }

    Ok((id_, StagedValueList {
        id: id_,
        name: name_.clone(),
        definition: StagedValueListDefinition::CustomValues(values)
    }))
}

pub fn parse_test(tokens: &[Token], info: &mut ParseInfo) -> Result<(u16, Test), CompileErr> {
    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?
        .value.parse::<u16>().expect("Unable to parse object id.");
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

pub fn parse_relation_criteria<'a>(tokens: &'a [Token], info: &mut ParseInfo) -> Result<StagedRelationCriteria, CompileErr> {
    let lhs_occ = expect(tokens, &vec![TokenType::Identifier], info)?;
    expect(tokens, &vec![TokenType::ScopeResolution], info)?;
    let lhs_field = expect(tokens, &vec![TokenType::Identifier], info)?;
    info.cursor += 1;
    let comparison_ = match tokens[info.cursor].ttype {
        TokenType::Eq => RelationComparison::Equal,
        TokenType::Neq => RelationComparison::NotEqual,
        TokenType::Gt => RelationComparison::Greater,
        TokenType::Gte => RelationComparison::GreaterEqual,
        TokenType::Lt => RelationComparison::Less,
        TokenType::Lte => RelationComparison::LessEqual,
        TokenType::Cartesian => RelationComparison::Cartesian,
        _ => return Err(CompileErr::UnexpectedToken { 
            token: tokens[info.cursor].clone(),
            expected: vec![TokenType::Eq, TokenType::Neq, TokenType::Gt,
            TokenType::Gte, TokenType::Lt, TokenType::Lte, TokenType::Cartesian] 
        })
    };
    let rhs_occ = expect(tokens, &vec![TokenType::Identifier], info)?;
    expect(tokens, &vec![TokenType::ScopeResolution], info)?;
    let rhs_field = expect(tokens, &vec![TokenType::Identifier], info)?;
    Ok(StagedRelationCriteria {
        occurrence1: lhs_occ.clone(),
        field1: lhs_field.clone(),
        occurrence2: rhs_occ.clone(),
        field2: rhs_field.clone(),
        comparison: comparison_,
    })
}

pub fn parse_relation(tokens: &[Token], info: &mut ParseInfo) -> Result<(u16, StagedRelation), CompileErr> {
    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?
        .value.parse::<u16>().expect("Unable to parse object id.");
    expect(tokens, &vec![TokenType::Assignment], info)?;
    let token = expect(tokens, &vec![TokenType::OpenBrace, TokenType::Identifier], info)?;

    let mut criterias_ = vec![];
    if token.ttype == TokenType::OpenBrace {
        while tokens.get(info.cursor).is_some() {
            // parse attribute
            let criteria = parse_relation_criteria(tokens, info)?;
            //if tables.iter().any(|t| t.is_none()) {
            //    tables[0] = Some(lhs_table);
            //    tables[1] = Some(rhs_table);
            //}
            //if !tables.iter().any(|search| search.unwrap() == lhs_table) {
            //    return Err(CompileErr::RelationCriteria { token: tokens[info.cursor - 2].clone() })
            //}
            //if !tables.iter().any(|search| search.unwrap() == rhs_table) {
            //    return Err(CompileErr::RelationCriteria { token: tokens[info.cursor].clone() })
            //}

            criterias_.push(criteria.clone());
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
                return Ok((id_, StagedRelation {
                    id: id_,
                    table1: criteria.occurrence1.value.to_string(),
                    table2: criteria.occurrence2.value.to_string(),
                    criterias: criterias_,
                }))
            }
        }
        unreachable!()
    } else {
        info.cursor -= 1;
        let criteria = parse_relation_criteria(tokens, info)?;
        criterias_.push(criteria.clone());
        Ok((id_, StagedRelation {
            id: id_,
            table1: criteria.occurrence1.value.to_string(),
            table2: criteria.occurrence2.value.to_string(),
            criterias: criterias_,
        }))
    }
}

pub fn parse_layout_attributes(tokens: &[Token], info: &mut ParseInfo) -> Result<Vec<LayoutFMAttribute>, CompileErr> {
    let attributes = vec![];
    while let Some(token) = tokens.get(info.cursor) {
        match token.ttype {
            TokenType::CloseBrace => {
                return Ok(attributes);
            }

            _ => {
                return Err(CompileErr::UnimplementedLanguageFeauture { 
                    feature: String::from("Layout Attributes"),
                    token: token.clone() 
                })
            }
        }
    }
    Ok(vec![])
}

pub fn parse_layout(tokens: &[Token], info: &mut ParseInfo) -> Result<(u16, StagedLayout), CompileErr> {
    let id_ = expect(tokens, &vec![TokenType::ObjectNumber], info)?
        .value.parse::<u16>().expect("Unable to parse object id.");
    let name_ = expect(tokens, &vec![TokenType::Identifier], info)?;


    expect(tokens, &vec![TokenType::Colon], info)?;

    let occurrence = expect(tokens, &vec![TokenType::Identifier], info)?;

    expect(tokens, &vec![TokenType::Assignment], info)?;
    expect(tokens, &vec![TokenType::OpenBrace], info)?;

    info.cursor += 1;
    let _attrs = parse_layout_attributes(tokens, info)?;

    Ok((id_, StagedLayout {
        id: id_,
        name: name_.clone(),
        base_occurrence: occurrence.clone(),
    }))
}

pub fn parse(tokens: &[Token]) -> Result<Stage, CompileErr> {
    let mut result = Stage::new();
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
            TokenType::Extern => {
                let (id, datasource) = parse_extern(tokens, &mut info)?;
                result.data_sources.insert(id, datasource);
            }
            TokenType::EOF => {
                break;
            }
            _ => { return Err(CompileErr::UnexpectedToken { 
                token: tokens[info.cursor].clone(), 
                expected: [
                    TokenType::Table, TokenType::TableOccurrence, TokenType::Relation,
                    TokenType::ValueList, TokenType::Script, TokenType::Test,
                    TokenType::EOF,
                ].to_vec(),
            }) }
        }
        if tokens[info.cursor].ttype == TokenType::EOF {
            break;
        }
        info.cursor += 1;
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs::read_to_string};

    use crate::{cadlang::{lexer::lex, token::{Location, Token, TokenType}}, schema::{DataType, RelationComparison, SerialTrigger, ValidationTrigger}};

    use crate::dbobjects::{data_source::*};

    use super::*;

    #[test]
    fn basic_table_parse_test() {
        let code = "
            table %1 Person = {
                field %1 id = {
                    datatype = Number,
                    required =true,
                    unique= true,
                    calculated_val = |get(uuid)|,
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

        for token in &tokens {
            println!("{:?}", token);
        }
        let schema = parse(&tokens).expect("Parsing failed.");
        let mut expected_fields = BTreeMap::new();
        expected_fields.insert(1, StagedField {
                    id: 1,
                    name: Token::with_value(
                        TokenType::Identifier,
                        Location { line: 3, column: 26 },
                        String::from("id")
                    ),
                    repetitions: 1,
                    dtype: DataType::Number,
                    global: false,
                    autoentry: StagedAutoEntry {
                        nomodify: false,
                        definition: StagedAutoEntryType::Calculation { 
                            code: String::from("get(uuid)"), 
                            noreplace: false, 
                        }
                    },
                    validation: StagedValidation {
                        checks: vec![
                            StagedValidationType::Required,
                            StagedValidationType::Unique
                        ],
                        message: String::from("Error with field validation."),
                        trigger: ValidationTrigger::OnEntry,
                        user_override: true,
                    }
                });
        expected_fields.insert(2, StagedField {
                    id: 2,
                    name: Token::with_value(
                        TokenType::Identifier,
                        Location { line: 10, column: 26 },
                        String::from("counter")
                    ),
                    dtype: DataType::Number,
                    repetitions: 1,
                    global: false,
                    autoentry: StagedAutoEntry {
                        nomodify: false,
                        definition: StagedAutoEntryType::Serial { 
                            next: 1, 
                            increment: 1, 
                            trigger: SerialTrigger::OnCreation 
                        }
                    },
                    validation: StagedValidation {
                        checks: vec![
                        ],
                        message: String::from("Error with field validation."),
                        trigger: ValidationTrigger::OnEntry,
                        user_override: true,
                    }
                });
        let expected = StagedTable {
            id: 1,
            name: Token::with_value(
                TokenType::Identifier,
                Location { line: 2, column: 22},
                "Person".to_string()),
            fields: expected_fields,
        };
        println!("{:?}\n", schema.tables[&1]);
        println!("{:?}\n", expected);
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

        let expected = StagedValueList {
            id: 1,
            name: Token::with_value(
                TokenType::Identifier,
                Location { line: 2, column: 23 },
                "basic".to_string()),
            definition: StagedValueListDefinition::CustomValues(vec![
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

        let expected = StagedValueList {
            id: 1,
            name: Token::with_value(
                TokenType::Identifier,
                Location { line: 2, column: 23 },
                "basic".to_string()),
            definition: StagedValueListDefinition::FromField { 
                occurrence: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 2, column: 31 },
                                String::from("Person_occ")
                            ),
                field1: Token::with_value(
                            TokenType::Identifier,
                            Location { line: 2, column: 43 },
                            "name".to_string()
                            ),
                field2: Some(Token::with_value(
                            TokenType::Identifier,
                            Location { line: 2, column: 61 },
                            "id".to_string()
                            )),
                from: None, 
                sort: StagedValueListSortBy::FirstField, 
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

        let expected = StagedValueList {
            id: 1,
            name: Token::with_value(
                TokenType::Identifier,
                Location { line: 2, column: 23 },
                "basic".to_string()),
            definition: StagedValueListDefinition::FromField { 
                occurrence: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 2, column: 31 },
                                String::from("Person_occ")
                            ),
                field1: Token::with_value(
                            TokenType::Identifier,
                            Location { line: 2, column: 43 },
                            "name".to_string()
                            ),
                field2: None,
                from: None, 
                sort: StagedValueListSortBy::FirstField, 
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
        for t in &tokens {
            println!("{:?}", t);
        }
        let schema = parse(&tokens).expect("Parsing failed.");

        let expected = StagedValueList {
            id: 1,
            name: Token::with_value(
                TokenType::Identifier,
                Location { line: 2, column: 23 },
                "basic".to_string()),
            definition: StagedValueListDefinition::FromField { 
                occurrence: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 2, column: 31 },
                                String::from("Person_occ")
                            ),
                field1: Token::with_value(
                            TokenType::Identifier,
                            Location { line: 2, column: 43 },
                            "name".to_string()
                            ),
                field2: None,
                from: Some(Token::with_value(
                    TokenType::Identifier,
                    Location { line: 3, column: 20 },
                    String::from("Salary_occ"))),
                sort: StagedValueListSortBy::SecondField, 
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

        let expected = StagedValueList {
            id: 1,
            name: Token::with_value(
                TokenType::Identifier,
                Location { line: 2, column: 23 },
                String::from("basic")),
            definition: StagedValueListDefinition::FromField { 
                occurrence: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 2, column: 31 },
                                String::from("Person_occ")
                            ),
                field1: Token::with_value(
                            TokenType::Identifier,
                            Location { line: 2, column:  43 },
                            String::from("name"),
                        ),
                field2: None,
                from: Some(Token::with_value(
                    TokenType::Identifier,
                    Location { line: 3, column: 20 }, 
                    String::from("Salary_occ"))), 
                sort: StagedValueListSortBy::FirstField, 
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

        let expected = StagedValueList {
            id: 1,
            name: Token::with_value(
                TokenType::Identifier,
                Location { line: 2, column: 23 },
                String::from("basic")),
            definition: StagedValueListDefinition::FromField { 
                occurrence: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 2, column: 31 },
                                String::from("Person_occ")
                            ),
                field1: Token::with_value(
                            TokenType::Identifier,
                            Location { line: 2, column: 43 },
                            String::from("name"),
                        ),
                field2: None,
                from: None,
                sort: StagedValueListSortBy::SecondField, 
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

        let expected_valuelist = StagedValueList {
            id: 1,
            name: Token::with_value(
                TokenType::Identifier,
                Location { line: 2, column: 23 },
                String::from("basic")),
            definition: StagedValueListDefinition::FromField { 
                occurrence: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 2, column: 31 },
                                String::from("Person_occ")
                            ),
                field1: Token::with_value(
                            TokenType::Identifier,
                            Location { line: 2, column:  43 },
                            String::from("name"),
                        ),
                field2: None,
                from: None,
                sort: StagedValueListSortBy::FirstField 
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

        let expected = StagedValueList {
            id: 1,
            name: Token::with_value(
                TokenType::Identifier,
                Location { line: 2, column: 23 },
                String::from("basic")),
            definition: StagedValueListDefinition::FromField { 
                occurrence: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 2, column: 31 },
                                String::from("Person_occ")
                            ),
                field1: Token::with_value(
                            TokenType::Identifier,
                            Location { line: 2, column:  43 },
                            String::from("name"),
                        ),
                field2: Some(Token::with_value(
                            TokenType::Identifier,
                            Location { line: 2, column:  61 },
                            String::from("id"),
                        )),
                from: Some(Token::with_value(
                    TokenType::Identifier,
                    Location { line: 3, column: 20 },
                    String::from("Salary_occ"))),
                sort: StagedValueListSortBy::SecondField, 
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

        let expected = StagedOccurrence {
            id: 1,
            data_source: None,
            name: Token::with_value(
                TokenType::Identifier,
                Location { line: 4, column: 33 },
                String::from("Person_occ")),
            base_table: Token::with_value(
                TokenType::Identifier,
                Location { line: 4, column: 46 },
                String::from("Person")),
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
        let expected = StagedRelation {
            id: 1,
            table1: String::from("Person_occ"),
            table2: String::from("Job_occ"),
            criterias: vec![StagedRelationCriteria {
                occurrence1: Token::with_value(
                            TokenType::Identifier,
                            Location { line: 2, column: 23 },
                            String::from("Person_occ")),
                field1: Token::with_value(
                            TokenType::Identifier,
                            Location { line: 2, column: 35 },
                            String::from("job_id")),
                comparison: RelationComparison::Equal,
                occurrence2: Token::with_value(
                            TokenType::Identifier,
                            Location { line: 2, column: 44 },
                            String::from("Job_occ")),
                field2: Token::with_value(
                            TokenType::Identifier,
                            Location { line: 2, column: 53 },
                            String::from("id")),
            }]
        };
        assert_eq!(expected, schema.relations[&1])
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
        let expected = StagedRelation {
            id: 1,
            table1: String::from("Person_occ"),
            table2: String::from("Job_occ"),
            criterias: vec![
                StagedRelationCriteria {
                    occurrence1: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 3, column: 13 },
                                String::from("Person_occ")),
                    field1: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 3, column: 25 },
                                String::from("job_id")),
                    comparison: RelationComparison::Equal,
                    occurrence2: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 3, column: 34 },
                                String::from("Job_occ")),
                    field2: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 3, column: 43 },
                                String::from("id")),
                },
                StagedRelationCriteria {
                    occurrence1: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 4, column: 13 },
                                String::from("Person_occ")),
                    field1: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 4, column: 25 },
                                String::from("first_name")),
                    comparison: RelationComparison::NotEqual,
                    occurrence2: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 4, column: 38 },
                                String::from("Job_occ")),
                    field2: Token::with_value(
                                TokenType::Identifier,
                                Location { line: 4, column: 47 },
                                String::from("name")),
                }
            ]
        };
        assert_eq!(expected, schema.relations[&1])
    }
    
    #[test]
    fn extern_basic() {
        let code = "
        extern %1 Quotes : \"quotes.cad\"
        ";
        let tokens = lex(code).expect("Tokenisation failed.");
        let schema = parse(&tokens).unwrap();

        let expected = DataSource {
            id: 1,
            name: String::from("Quotes"),
            paths: vec![String::from("quotes.cad")],
            dstype: DataSourceType::Cadmus,
        };
        assert_eq!(expected, schema.data_sources[&1]);

    }

    #[test]
    fn compile_initial_cad() {
        let code = read_to_string("test_data/cad_files/initial.cad").expect("Unable to read file.");

        let tokens = lex(&code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.");
    }
}
