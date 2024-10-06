
use crate::{burn_script, fm_script_engine::fm_script_engine_instructions::Script};
use super::token::*; 
use crate::schema::*;

pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(ts: Vec<Token>) -> Self {
        Self {
            tokens: ts,
        }
    }

    pub fn parse_program(&self) -> Result<Schema, String> {

        let mut ret = Schema::new();
        let mut parser_iter = self.tokens.iter().peekable().clone();

        while let Some(t) = parser_iter.next() {
            match t.ttype {
                TokenType::Table => {
                    let mut table = Table::new(ret.tables.len());
                    if parser_iter.peek().unwrap().ttype != TokenType::Identifier {
                        return Err("Expected Table name after \"table\" keyword.".to_string())
                    } else {
                        table.name = parser_iter.next().unwrap().text.clone();
                    }
                    while let Some(mut n) = parser_iter.next() {
                        match n.ttype {
                            TokenType::End => 
                            {
                                n = parser_iter.next().unwrap();
                                if n.ttype == TokenType::Table {
                                    if parser_iter.next().unwrap().ttype != TokenType::SemiColon {
                                        return Err("Please end top level constructs with \";\"".to_string());
                                    }
                                    break;
                                } else {
                                    return Err(std::format!("Unexpected {} after \"end\"", n.text).to_string());
                                }
                            }
                            _ => {
                                continue;
                            }
                        }
                    }
                    ret.tables.insert(ret.tables.len() + 1, table);
                },
                TokenType::Relationship => {
                    let mut relationship = Relation::new(ret.relations.len());
                    if parser_iter.peek().unwrap().ttype != TokenType::Colon {
                        return Err("Expected colon after \"relationship\" keyword.".to_string())
                    } 
                    while let Some(mut n) = parser_iter.next() {
                        match n.ttype {
                            TokenType::Identifier => {
                                if parser_iter.peek().unwrap().ttype == TokenType::Colon {
                                    relationship.table1_name = n.text.clone();
                                    parser_iter.next();
                                    let n = parser_iter.next().unwrap();
                                    relationship.table2_name = n.text.clone();
                                } else {

                                }
                            }
                            TokenType::End => 
                            {
                                n = parser_iter.next().unwrap();
                                if n.ttype == TokenType::Relationship {
                                    if parser_iter.next().unwrap().ttype != TokenType::SemiColon {
                                        return Err("Please end top level constructs with \";\"".to_string());
                                    }
                                    break;
                                } else {
                                    return Err(std::format!("Unexpected {} after \"end\"", n.text).to_string());
                                }
                            }
                            _ => {
                                continue;
                            }
                        }
                    }
                    ret.relations.insert(ret.tables.len() + 1, relationship);
                },
                TokenType::ValueList => {
                    let mut value_list = ValueList::new(ret.value_lists.len());
                    if parser_iter.peek().unwrap().ttype != TokenType::Identifier {
                        return Err("Expected identifier after \"value_list\" keyword.".to_string())
                    } else {
                        value_list.name = parser_iter.next().unwrap().text.clone();
                    }
                    while let Some(mut n) = parser_iter.next() {
                        match n.ttype {
                            TokenType::End => 
                            {
                                n = parser_iter.next().unwrap();
                                if n.ttype == TokenType::ValueList {
                                    if parser_iter.next().unwrap().ttype != TokenType::SemiColon {
                                        return Err("Please end top level constructs with \";\"".to_string());
                                    }
                                    break;
                                } else {
                                    return Err(std::format!("Unexpected {} after \"end\"", n.text).to_string());
                                }
                            }
                            _ => {
                                continue;
                            }
                        }
                    }
                    ret.value_lists.insert(ret.value_lists.len() + 1, value_list);
                },
                TokenType::Script => {
                    let scripts: Vec<Script> = vec![];
                    if parser_iter.peek().unwrap().ttype != TokenType::Colon {
                        return Err("Expected Colon after top level declaration script keyword.".to_string());
                    }
                    parser_iter.next();
                    if parser_iter.peek().unwrap().ttype != TokenType::OpenSquare {
                        return Err("Please use \"[\" to denote entering a script block.".to_string());
                    }
                    parser_iter.next();
                    let block = parser_iter.next().unwrap();

                    let scripts = burn_script::compiler::BurnScriptCompiler::compile_burn_script(&block.text);
                    
                    for s in scripts {
                        ret.scripts.insert(ret.scripts.len() + 1, s);
                    }

                    let next = parser_iter.next().unwrap();
                    if next.ttype != TokenType::CloseSquare {
                        return Err("Please end BurnScript block with \"]\"".to_string());
                    }
                },
                TokenType::TableOccurence => {
                    let mut table_occurence = TableOccurrence::new(ret.table_occurrences.len());
                    if parser_iter.peek().unwrap().ttype != TokenType::Identifier {
                        return Err("Expected Table name after \"table\" keyword.".to_string())
                    } else {
                        table_occurence.name = parser_iter.next().unwrap().text.clone();
                    }
                    while let Some(mut t) = parser_iter.next() {
                        // println!("{:?}", t.ttype);
                        match t.ttype {
                            TokenType::Table => {
                                t = parser_iter.next().unwrap();
                                if t.ttype != TokenType::Colon {
                                    return Err("Expected colon after table specifier.".to_string());
                                }
                                t = parser_iter.next().unwrap();
                                if t.ttype != TokenType::Identifier {
                                    return Err("Expected Identifier for table reference.".to_string());
                                }
                                table_occurence.table_actual_name = t.text.clone();
                            },
                            TokenType::Comma => {

                            },
                            TokenType::End => {
                                t = parser_iter.next().unwrap();
                                if t.ttype == TokenType::TableOccurence {
                                    if parser_iter.next().unwrap().ttype != TokenType::SemiColon {
                                        return Err("Please end top level constructs with \";\"".to_string());
                                    }
                                    break;
                                } else {
                                    return Err(std::format!("Unexpected {} after \"end\"", t.text).to_string());
                                }
                            }
                            _ => {

                            }
                        }
                    }
                    ret.table_occurrences.insert(ret.table_occurrences.len(), table_occurence);
                },
                TokenType::Test => {
                    let mut test = Test::new();
                    if parser_iter.peek().unwrap().ttype != TokenType::Identifier {
                        return Err("Expected identifier after \"test\" keyword.".to_string())
                    } else {
                        test.name = parser_iter.next().unwrap().text.clone();
                    }

                    while let Some(mut n) = parser_iter.next() {
                        match n.ttype {
                            TokenType::Script => {
                                if parser_iter.next().unwrap().ttype != TokenType::Colon {
                                    return Err("expected ':' after object block".to_string());
                                }
                                if parser_iter.next().unwrap().ttype != TokenType::OpenSquare {
                                    return Err("expected '[' to start script block".to_string());
                                }
                                let block = parser_iter.next();
                                let scripts = burn_script::compiler::BurnScriptCompiler::compile_burn_script(&block.unwrap().text);
                                if scripts.len() > 1 {
                                    return Err("Please ensure that tests only have 1 script defined.".to_string());
                                }
                                test.script = scripts[0].clone();
                                continue;
                            }
                            TokenType::End => 
                            {
                                n = parser_iter.next().unwrap();
                                if n.ttype == TokenType::Test {
                                    if parser_iter.next().unwrap().ttype != TokenType::SemiColon {
                                        return Err("Please end top level constructs with \";\"".to_string());
                                    }
                                    break;
                                } else {
                                    return Err(std::format!("Unexpected {} after \"end\"", n.text).to_string());
                                }
                            }
                            _ => {
                                continue;
                            }
                        }
                    }
                    ret.tests.insert(ret.tests.len(), test);
                },
                _ => {
                    return Err("Unrecognized top level structure.".to_string());
                }
            }
        }
        return Ok(ret);
    }
}

