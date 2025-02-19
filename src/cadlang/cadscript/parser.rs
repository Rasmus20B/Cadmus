
use crate::dbobjects::{calculation::{Calculation, CalculationString}, scripting::{instructions::Instruction, arguments::*, script::Script}};
use super::token::TokenVal;
use super::proto_instruction::{ProtoInstruction, ProtoLayoutSelection};

use std::collections::HashMap;
use super::arg_lookups::ARG_LOOKUP;

pub enum ArgFormat {
    Positional,
    Labeled,
}

#[derive(Clone, Debug)]
pub struct ArgKeyVal {
    label: String,
    value: TokenVal,
}

impl ArgKeyVal {
    pub fn new(label_: String, value_: TokenVal) -> Self {
        Self {
            label: label_,
            value: value_,
        }
    }
}

#[derive(Debug)]
pub enum ParseErr {
    UnexpectedToken { token: TokenVal, expected: Vec<TokenVal> },
    PositionalLabelMix,
    InvalidLabel,
    UnexpectedEOF,
    UnexpectedScopeCloser,
}

pub struct ParseInfo {
    cursor: usize,
}

fn expect<'a>(tokens: &'a [TokenVal], expected_: &Vec<TokenVal>, info: &mut ParseInfo) -> Result<&'a TokenVal, ParseErr> {
    if let Some(token) = tokens.get(info.cursor) {
        let t_type = std::mem::discriminant(token);

        for t_expected in expected_ {
            if t_type == std::mem::discriminant(t_expected) {
                info.cursor += 1;
                return Ok(token);
            }
        }
        info.cursor += 1;
        return Err(ParseErr::UnexpectedToken { 
            token: token.clone(), 
            expected: expected_.to_vec(),
        });
    } else {
        info.cursor += 1;
        Err(ParseErr::UnexpectedEOF)
    }
}

pub fn parse_labeled_arg<'a>(label: &'a TokenVal, tokens: &'a [TokenVal], info: &mut ParseInfo) -> Result<ArgKeyVal, ParseErr> {
    expect(tokens, &vec![TokenVal::Assignment], info)?;

    let value = expect(tokens, &vec![TokenVal::KeywordArg(String::new()),
        TokenVal::StringArg(String::new()),
        TokenVal::NumberArg(0.0),
        TokenVal::CalculationArg(String::new())
    ],
    info)?;

    return Ok(ArgKeyVal::new(label.get_value().unwrap(), value.clone()))
}

pub fn parse_labeled_args<'a>(tokens: &'a [TokenVal], info: &mut ParseInfo) -> Result<Vec<ArgKeyVal>, ParseErr> {
    let mut args = vec![];
    Ok(args)
}

pub fn parse_positional_args<'a>(instr: &'a str, tokens: &'a [TokenVal], info: &mut ParseInfo) -> Result<Vec<ArgKeyVal>, ParseErr> {

    let mut arg_n = 1;
    let mut args = vec![];

    while info.cursor < tokens.len() {
        let val = expect(tokens, &vec![
            TokenVal::CloseParen,
            TokenVal::Identifier(String::new()),
            TokenVal::KeywordArg(String::new()),
            TokenVal::StringArg(String::new()),
            TokenVal::NumberArg(0.0),
            TokenVal::FieldReference(String::new(), String::new()),
            TokenVal::CalculationArg(String::new())
        ],
        info)?;

        let tmp_pair = match val {
            TokenVal::CalculationArg(..) 
                | TokenVal::Variable(..)
                | TokenVal::Identifier(..)
                | TokenVal::KeywordArg(..)
                | TokenVal::StringArg(..)
                | TokenVal::FieldReference(..) => ArgKeyVal {
                label: ARG_LOOKUP.get_argname(instr, arg_n).unwrap().to_string(),
                value: val.clone(),
            },
            TokenVal::ArgLabel(label) => return Err(ParseErr::PositionalLabelMix),
            TokenVal::CloseParen => { break; }
            _ => { eprintln!("{:?}", val); unreachable!() }
        };
        println!("{} :: {:?}", arg_n, tmp_pair);
        args.push(tmp_pair);
        arg_n += 1;
    }

    Ok(args)
}

pub fn parse_first_arg<'a>(instr: &'a str, tokens: &'a [TokenVal], info: &mut ParseInfo) -> Result<Option<(ArgFormat, ArgKeyVal)>, ParseErr> {
    let name_or_val = expect(tokens, &vec![
        TokenVal::CloseParen,
        TokenVal::Identifier(String::new()),
        TokenVal::KeywordArg(String::new()),
        TokenVal::StringArg(String::new()),
        TokenVal::NumberArg(0.0),
        TokenVal::CalculationArg(String::new()),
        TokenVal::FieldReference(String::new(), String::new()),
    ],
    info)?;

    if [TokenVal::CloseParen, TokenVal::OpenBrace].contains(name_or_val) {
        return Ok(None);
    }

    match expect(tokens, &vec![TokenVal::Comma, TokenVal::Assignment, TokenVal::CloseParen], info)? {
        TokenVal::Comma | TokenVal::CloseParen => {
            let name = match ARG_LOOKUP.get_argname(instr, 0) {
                Some(name) => name,
                None => return Err(ParseErr::InvalidLabel)
            };
            Ok(Some((ArgFormat::Positional, ArgKeyVal::new(name.to_string(), name_or_val.clone()))))
        }
        TokenVal::Assignment => {
            let value = expect(tokens, &vec![TokenVal::KeywordArg(String::new()),
                TokenVal::StringArg(String::new()),
                TokenVal::NumberArg(0.0),
                TokenVal::CalculationArg(String::new())
            ],
            info)?;

            Ok(Some((ArgFormat::Labeled, ArgKeyVal::new(name_or_val.get_value().unwrap(), value.clone()))))
        }
        _ => unreachable!()
    }
}

pub fn parse_args<'a>(instr: &'a str, tokens: &'a [TokenVal], info: &mut ParseInfo) -> Result<Vec<ArgKeyVal>, ParseErr> {
    // Return an empty Vec early if there are no args.
    let early = expect(&tokens, &vec![TokenVal::OpenParen, TokenVal::OpenBrace], info)?;
    if *early == TokenVal::OpenBrace { return Ok(vec![]) }

    let mut result = vec![];

    match parse_first_arg(instr, tokens, info).unwrap() {
        Some((format, first_arg)) => {
            result.push(first_arg);

            if tokens[info.cursor - 1] != TokenVal::CloseParen {
                result.extend(match format {
                    ArgFormat::Positional => parse_positional_args(instr, tokens, info)?,
                    ArgFormat::Labeled => parse_labeled_args(tokens, info)?,
                });
            }

            Ok(result)
        },
        None => {
            return Ok(vec![]);
        }
    }
}

pub fn parse(tokens: Vec<TokenVal>) -> Result<Vec<ProtoInstruction>, ParseErr> {
    for tok in &tokens {
        println!("{:?}", tok);
    }
    let mut info = ParseInfo { cursor: 0 };
    let mut instructions = vec![];
    let mut scope_stack = vec![TokenVal::EOF];
    while info.cursor < tokens.len() {
        println!("cursor: {} :: {:?}", info.cursor, tokens[info.cursor]);
        let instr = match expect(&tokens, &vec![TokenVal::Identifier(String::new()), TokenVal::OpenBrace, TokenVal::CloseBrace], &mut info)? {
            TokenVal::Identifier(inner) => inner,
            TokenVal::OpenBrace => continue,
            TokenVal::CloseBrace => {
                match scope_stack.last() {
                    Some(TokenVal::Loop) => { 
                        instructions.push(ProtoInstruction::EndLoop); 
                        scope_stack.pop();
                        continue 
                    },
                    Some(TokenVal::EOF) => {
                        break;
                    }
                    None => {
                        return Err(ParseErr::UnexpectedScopeCloser)
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!()
        };

        let arguments = parse_args(instr, &tokens, &mut info).unwrap();
        instructions.push(match instr.as_str() {
            "set_variable" => {
                let name = arguments.iter().find(|arg| arg.label == "var").unwrap();
                let val = arguments.iter().find(|arg| arg.label == "expr").unwrap();
                let rep = arguments.iter().find(|arg| arg.label == "rep");
                let rep = match rep {
                    Some(rep) => rep,
                    None => &ArgKeyVal { 
                        label: String::from("rep"),
                        value: TokenVal::CalculationArg(String::from("1")) 
                    },
                };

                ProtoInstruction::SetVariable { 
                    name: name.value.get_value().unwrap(),
                    value: CalculationString(val.value.get_value().unwrap()),
                    repetition: CalculationString(rep.value.get_value().unwrap()), 
                }
            },
            "loop" => { 
                scope_stack.push(TokenVal::Loop); 
                ProtoInstruction::Loop 
            },
            "exit_loop_if" => ProtoInstruction::ExitLoopIf { condition: CalculationString(arguments.iter()
                .find(|arg| arg.label == "expr")
                .unwrap().value
                    .get_value()
                    .unwrap()
                    .clone()) 
            },
            "new_record" | "new_request" => ProtoInstruction::NewRecordRequest,
            "go_to_layout" => {
                let layout_ = match &arguments.iter().find(|arg| arg.label == "layout").unwrap().value {
                    TokenVal::CalculationArg(calc) => ProtoLayoutSelection::Calculation(CalculationString(calc.to_string())),
                    TokenVal::Identifier(ident) => ProtoLayoutSelection::UnresolvedName(ident.to_string()),
                    _ => unreachable!()
                };
                let animation = match arguments.iter().find(|arg| arg.label == "animation") {
                    Some(inner) => inner,
                    None => &ArgKeyVal { label: "animation".to_string(), value: TokenVal::KeywordArg("None".to_string()) },
                };

                let animation_encoded = match animation.value.get_value().unwrap().as_str() {
                    "None" => LayoutAnimation::None,
                    _ => LayoutAnimation::None,
                };

                ProtoInstruction::GoToLayout { layout: layout_, animation: animation_encoded }

            }
            _ => ProtoInstruction::Print
        });

    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::super::{token::*, proto_instruction::ProtoInstruction, *};
    use crate::dbobjects::{scripting::arguments::*, calculation::CalculationString};
    #[test]
    fn basic_parse() {
        let code = "set_variable($x, |0|)
                    go_to_layout(Person)
                    loop {
                      exit_loop_if(|$x == 10|)
                      new_record()
                      set_variable($x, |$x + 1|)
                    }";

        let tokens = lexer::lex(code);

        let instrs = parser::parse(tokens).unwrap();

        let expected = vec![
            ProtoInstruction::SetVariable { 
                name: String::from("$x"),
                value: CalculationString("0".to_string()),
                repetition: CalculationString("1".to_string()),
            },
            ProtoInstruction::GoToLayout {
                layout: parser::ProtoLayoutSelection::UnresolvedName("Person".to_string()),
                animation: LayoutAnimation::None,
            },
            ProtoInstruction::Loop,
            ProtoInstruction::ExitLoopIf { condition: CalculationString("$x == 10".to_string()) },
            ProtoInstruction::NewRecordRequest,
            ProtoInstruction::SetVariable { 
                name: "$x".to_string(),
                value: CalculationString("$x + 1".to_string()),
                repetition: CalculationString("1".to_string()) 
            },
        ];

        assert_eq!(instrs.len(), expected.len());

        for (actual, expected) in instrs.iter().zip(&expected) {
            println!("{:?}", actual);
            //assert_eq!(actual, expected);
        }
    }

    fn basic_parse_labels() {
        let code = "set_variable(variable=$x, expr=|0|)
                    go_to_layout(layout=\"Person\")
                    loop {
                      exit_loop_if(expr=|$x == 10|)
                      new_record()
                      set_variable(variable=$x, expr=|$x + 1|)
                    }";

        let tokens = lexer::lex(code);
        let instrs = parser::parse(tokens).unwrap();
    }
}
