
use super::command::*;

#[derive(Debug)]
pub enum ParseErr {
    EmptyLine,
    UnknownCommand(String),
    IncorrectNArguments(String, u8, u8),
}


pub struct Parser {
}

impl Parser {
    fn lex(&self, command: &str) -> Vec<Token> {
        command.split_whitespace()
            .map(|lexeme| match lexeme {
                "run" => Token::Run,
                "open" => Token::Open,
                "quit" => Token::Quit,
                s if s.parse::<u32>().is_ok() => Token::IntLiteral(s.parse::<u32>().unwrap()),
                s => Token::String(s.to_string())
            })
        .collect()
    }

    pub fn parse(&mut self, code: &str) -> Result<Command, ParseErr> {
        let tokens = self.lex(code);
        if tokens.is_empty() { return Err(ParseErr::EmptyLine) }

        match tokens[0] {
            Token::Run => {
                if tokens.len() != 3 {
                    return Err(ParseErr::IncorrectNArguments("run".to_string(), tokens.len() as u8, 2));
                }
                Ok(Command::Run { file_name: tokens[2].as_string(), test_name: tokens[1].as_string() })
            },
            _ => return Err(ParseErr::UnknownCommand(format!("{:?}", tokens[0])))
        }
    }
}

#[derive(Debug)]
pub(crate) enum Token {
    String(String),
    IntLiteral(u32),

    Run,
    Open,
    Quit,
}

impl Token {
    pub fn as_string(&self) -> String {
        match self {
            Token::String(string) => string.to_string(),
            Token::IntLiteral(n) => n.to_string(),
            Token::Run => "run".to_string(),
            Token::Open => "open".to_string(),
            Token::Quit => "quit".to_string(),
        }
    }
}
