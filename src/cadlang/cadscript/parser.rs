
use crate::dbobjects::scripting::{instructions::Instruction, arguments::*, script::Script};
use super::token::Token;

use super::arg_lookups::ARG_LOOKUP;

pub enum ArgFormat {
    Positional,
    Labeled,
}

pub struct ArgKeyVal {
    label: Token,
    value: Token,
}

impl ArgKeyVal {
    pub fn new(label_: Token, value_: Token) -> Self {
        Self {
            label: label_,
            value: value_,
        }
    }
}

pub enum ParseErr {
    UnexpectedToken { token: Token, expected: Vec<Token> },
    PositionalLabelMix,
    InvalidLabel,
    UnexpectedEOF,
}

pub struct ParseInfo {
    cursor: usize,
}

fn expect<'a>(tokens: &'a [Token], expected_: &Vec<Token>, info: &mut ParseInfo) -> Result<&'a Token, ParseErr> {
    info.cursor += 1;
    if let Some(token) = tokens.get(info.cursor) {
        let t_type = std::mem::discriminant(token);

        for t_expected in expected_ {
            if t_type == std::mem::discriminant(t_expected) {
                return Ok(token);
            }
        }
        return Err(ParseErr::UnexpectedToken { 
            token: token.clone(), 
            expected: expected_.to_vec(),
        });
    } else {
        Err(ParseErr::UnexpectedEOF)
    }
}

pub fn parse_labeled_arg<'a>(label: &'a Token, tokens: &'a [Token], info: &mut ParseInfo) -> Result<ArgKeyVal, ParseErr> {
    expect(tokens, &vec![Token::Assignment], info)?;

    let value = expect(tokens, &vec![Token::KeywordArg(String::new()),
        Token::StringArg(String::new()),
        Token::NumberArg(0.0),
        Token::CalculationArg(String::new())
    ],
    info)?;

    return Ok(ArgKeyVal::new(label.clone(), value.clone()))
}

pub fn parse_labeled_args<'a>(tokens: &'a [Token], info: &mut ParseInfo) -> Result<Vec<ArgKeyVal>, ParseErr> {
    let mut args = vec![];

    Ok(args)
}

pub fn parse_first_arg<'a>(instr: &'a str, tokens: &'a [Token], info: &mut ParseInfo) -> Result<(ArgFormat, ArgKeyVal), ParseErr> {
    let name_or_val = expect(tokens, &vec![
        Token::Identifier(String::new()),
        Token::KeywordArg(String::new()),
        Token::StringArg(String::new()),
        Token::NumberArg(0.0),
        Token::CalculationArg(String::new())
    ],
    info)?;

    match expect(tokens, &vec![Token::Comma, Token::Assignment], info)? {
        Token::Comma => {
            let value = match name_or_val {
                Token::Identifier(inner) => inner,
                _ => unreachable!()
            };
            let name = match ARG_LOOKUP.get_argname(instr, 0) {
                Some(name) => name,
                None => return Err(ParseErr::InvalidLabel)
            };
            Ok((ArgFormat::Positional, ArgKeyVal::new(Token::Identifier(name.to_string()), name_or_val.clone())))
        }
        Token::Assignment => {
            let value = expect(tokens, &vec![Token::KeywordArg(String::new()),
                Token::StringArg(String::new()),
                Token::NumberArg(0.0),
                Token::CalculationArg(String::new())
            ],
            info)?;

            Ok((ArgFormat::Labeled, ArgKeyVal::new(name_or_val.clone(), value.clone())))
        }
        _ => unreachable!()
    }
}

pub fn parse_args<'a>(instr: &'a str, tokens: &'a [Token], info: &mut ParseInfo) -> Result<Vec<ArgKeyVal>, ParseErr> {
    expect(&tokens, &vec![Token::OpenParen], info)?;
    let mut result = vec![];

    let (format, first_arg) = parse_first_arg(instr, tokens, info)?;
    result.push(first_arg);

    result.extend(match format {
        ArgFormat::Positional => todo!(),
        ArgFormat::Labeled => parse_labeled_args(tokens, info)?,
    });

    Ok(result)
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Instruction>, ParseErr> {
    let mut info = ParseInfo { cursor: 0 };
    let instructions = vec![];
    while info.cursor < tokens.len() {
        let instr = match expect(&tokens, &vec![Token::Identifier(String::new())], &mut info)? {
            Token::Identifier(inner) => inner,
            _ => unreachable!()
        };
        let arguments = parse_args(instr, &tokens, &mut info);
    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::super::*;
    #[test]
    fn basic_parse() {
        let code = "set_variable($x, |0|);
                    go_to_layout(\"Person\");
                    loop {
                      exit_loop_if(|$x == 10|);
                      new_record_request();
                      set_variable($x, |$x + 1|);
                    }";

        let tokens = lexer::lex(code);
    }

    fn basic_parse_labels() {
        let code = "set_variable(variable=$x, expr=|0|);
                    go_to_layout(layout=\"Person\");
                    loop {
                      exit_loop_if(expr=|$x == 10|);
                      new_record_request();
                      set_variable(variable=$x, expr=|$x + 1|);
                    }";

        let tokens = lexer::lex(code);
    }
}
