use std::{str::FromStr};


use crate::{schema::Script, fm_script_engine::fm_script_engine_instructions::{Instruction, ScriptStep}};

use super::tokens::{Token, TokenType};


#[derive(Debug, PartialEq)]
pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(toks: Vec<Token>) -> Self {
        Self {
            tokens: toks,
        }
    }

    pub fn parse(&self) -> Result<Vec<Script>, &str> {
        let mut scripts = vec![];
        let mut parser_iter = self.tokens.iter().peekable();

        let mut punc_stack: Vec<Instruction> = vec![];

        while let Some(t) = parser_iter.next() {
            match t.ttype {
                TokenType::Define => {
                    let mut tmp = Script::new();
                    let peeked = parser_iter.next().unwrap();
                    if peeked.ttype == TokenType::Identifier {
                        tmp.name = peeked.value.clone();
                    }

                    /* Parse arguments to script */
                    while let Some(t) = parser_iter.next() {
                        match t.ttype {
                            TokenType::Argument => {
                                tmp.arguments.push(t.value.clone());
                            },
                            TokenType::Comma => {
                                continue;
                            },
                            TokenType::OpenParen => {
                                continue;
                            }
                            TokenType::CloseParen => {
                                continue;
                            },
                            TokenType::OpenBracket => {
                                break;
                            }
                            _ => { eprintln!("invalid token."); }
                        };
                    }

                    /* Parse the instructions inside the script with their options */
                    while let Some(t) = parser_iter.next() {
                        match t.ttype {
                            TokenType::Identifier => {
                                if let Ok(op) = Instruction::from_str(&t.value) {
                                    let mut step = ScriptStep {
                                        opcode: op,
                                        index: 0,
                                        switches: vec![],
                                    };

                                    while let Some(t) = parser_iter.next() {
                                        match t.ttype {
                                            TokenType::Argument => {
                                                step.switches.push(t.value.clone());
                                            }
                                            _ => {
                                                if t.ttype != TokenType::SemiColon {
                                                    panic!("unterminated script step: {:?}", t.ttype);
                                                }
                                                tmp.instructions.insert(tmp.instructions.len(), step);
                                                break;
                                            }
                                        }
                                    }
                                } else {
                                    eprintln!("Invalid script step: {}", t.value);
                                }
                            },

                            TokenType::CloseBracket => {
                                let top = punc_stack.last();
                                if top.is_none() {
                                    break;
                                }
                                let n = parser_iter.peek();
                                if n.is_some() {
                                    match n.unwrap().ttype {
                                        TokenType::Else => {
                                            continue;
                                        }
                                        TokenType::Elif => {
                                            continue;
                                        }
                                        _ => {}
                                    }
                                }
                                let op: Instruction;
                                match top.unwrap() {
                                    Instruction::If => {
                                        op = Instruction::EndIf;
                                    },
                                    Instruction::Loop => {
                                        op = Instruction::EndLoop;
                                    },
                                    _ => {
                                        return Err("Invalid scope signifier.");
                                    }
                                }
                                punc_stack.pop();
                                let step = ScriptStep {
                                    opcode: op,
                                    index: 0,
                                    switches: vec![],
                                };
                                tmp.instructions.insert(tmp.instructions.len(), step);
                            },
                            TokenType::OpenBracket => {},
                            TokenType::Loop => {
                                let check = parser_iter.next();
                                if check.unwrap().ttype != TokenType::OpenBracket {
                                    eprintln!("Expected '{{' after loop.");
                                    break;
                                }
                                let step = ScriptStep {
                                    opcode: Instruction::Loop,
                                    index: 0,
                                    switches: vec![],
                                };
                                tmp.instructions.insert(tmp.instructions.len(), step);
                                punc_stack.push(Instruction::Loop);
                            },
                            TokenType::Elif => {
                                let mut step = ScriptStep {
                                    opcode: Instruction::ElseIf,
                                    index: 0,
                                    switches: vec![]
                                };

                                while let Some(t) = parser_iter.next() {
                                    match t.ttype {
                                        TokenType::Argument => {
                                            step.switches.push(t.value.clone());
                                        }
                                        _ => {
                                            if t.ttype != TokenType::OpenBracket {
                                                panic!("missing open bracket on elif");
                                            }
                                            tmp.instructions.insert(tmp.instructions.len(), step);
                                            break;
                                        }
                                    }
                                }
                            },
                            TokenType::If => {
                                let mut step = ScriptStep {
                                    opcode: Instruction::If,
                                    index: 0,
                                    switches: vec![]
                                };
                                while let Some(t) = parser_iter.next() {
                                    match t.ttype {
                                        TokenType::Argument => {
                                            step.switches.push(t.value.clone());
                                        }
                                        _ => {
                                            if t.ttype != TokenType::OpenBracket {
                                                panic!("Missing open bracket on if");
                                            }
                                            tmp.instructions.insert(tmp.instructions.len(), step);
                                            break;
                                        }
                                    }
                                }
                                punc_stack.push(Instruction::If);
                            },
                            TokenType::Else => {
                                let check = parser_iter.next();
                                if check.unwrap().ttype != TokenType::OpenBracket {
                                    eprintln!("Expected '{{' after loop.");
                                    break;
                                }
                                let step = ScriptStep {
                                    opcode: Instruction::Else,
                                    index: 0,
                                    switches: vec![]
                                };
                                tmp.instructions.insert(tmp.instructions.len(), step);
                            },
                            _ => { 
                                eprintln!("Invalid token in script: {:?}", t); 
                            }
                        }
                    }
                    scripts.push(tmp);
                },
                _ => {
                    eprintln!("Unrecognized token");
                }
            }
        }
        return Ok(scripts);
    }
}

#[cfg(test)]
mod tests {
    use crate::{burn_script::lexer, fm_script_engine::fm_script_engine_instructions::{Instruction, ScriptStep}};

    use super::Parser;


    #[test]
    pub fn parse_test_basic() {
        let code = "
        define test_func(x, y) {
            set_variable(i, x);
            loop {
                exit_loop_if(i == y);
                set_variable(i, (i + 1));
                if(i == 7) {
                    set_variable(x, 20);
                } else {
                    set_variable(x, \"Jeff\" & \" Keighly\");
                }
            }
            assert(1 == 1);
            exit_script(i);
        }";
        let tokens = lexer::Lexer::new(code.to_string()).get_tokens();
        let scripts = Parser::new(tokens).parse().expect("Unable to parse token stream");

        let handle = scripts[0].clone();
        assert_eq!(handle.name, "test_func");
        assert_eq!(handle.arguments, vec!["x", "y"]);

        let steps_actual = vec![
            ScriptStep { opcode: Instruction::SetVariable,
                         switches: vec!["i".to_string(), "x".to_string()],
                         index: 0,
            },
            ScriptStep { opcode: Instruction::Loop,
                         switches: vec![],
                         index: 0,
            },
            ScriptStep { opcode: Instruction::ExitLoopIf,
                         switches: vec!["i == y".to_string()],
                         index: 0,
            },
            ScriptStep { opcode: Instruction::SetVariable,
                         switches: vec!["i".to_string(), "(i + 1)".to_string()],
                         index: 0,
            },
            ScriptStep { opcode: Instruction::If,
                         switches: vec!["i == 7".to_string()],
                         index: 0,
            },
            ScriptStep { opcode: Instruction::SetVariable,
                         switches: vec!["x".to_string(), "20".to_string()],
                         index: 0,
            },
            ScriptStep { opcode: Instruction::Else,
                         switches: vec![],
                         index: 0,
            },
            ScriptStep { opcode: Instruction::SetVariable,
                         switches: vec!["x".to_string(), "\"Jeff\" & \" Keighly\"".to_string()],
                         index: 0,
            },
            ScriptStep { opcode: Instruction::EndIf,
                         switches: vec![],
                         index: 0,
            },
            ScriptStep { opcode: Instruction::EndLoop,
                         switches: vec![],
                         index: 0,
            },
            ScriptStep { opcode: Instruction::Assert,
                         switches: vec!["1 == 1".to_string()],
                         index: 0,
            },
            ScriptStep { opcode: Instruction::ExitScript,
                         switches: vec!["i".to_string()],
                         index: 0,
            },
        ];
        for (i, step) in steps_actual.iter().enumerate() {
            assert_eq!(*step, handle.instructions.get(i).unwrap().clone());
        }
    }
}
