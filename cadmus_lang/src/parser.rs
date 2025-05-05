use crate::diagnostic::Diagnostic;
use crate::error::Result;
use crate::token::Token;
use crate::keyvalue::{BlockValue, CadmusObject, Key, KeyValueBlock, KeyValueEntry};

pub enum TopLevelObject {
    Table {
        id: u32,
        name: String,
        entries: KeyValueBlock,
    },
    Field {
        id: u32,
        name: String,
        entries: KeyValueBlock,
    }
}

struct Parser<'a> {
    stream: std::iter::Peekable<std::slice::Iter<'a, Token>>,
    current: Option<&'a Token>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            stream: tokens.iter().peekable(),
            current: None,
        }
    }

    pub fn next(&mut self) -> Option<&&Token> {
        self.current = self.stream.next();
        self.current.as_ref()
    }

    pub fn peek(&mut self) -> Option<&&Token> {
        self.stream.peek()
    }
}

fn parse(tokens: &[Token], diagnostics: &mut Vec<Diagnostic>) -> Result<KeyValueBlock> {
    use crate::token::TokenValue;
    let mut parser = Parser::new(tokens);
    let mut entries = KeyValueBlock::new();

    while let Some(token) = parser.next() {
        let tmp = match token.token_val {
            TokenValue::Table => {
                // parse top level decl,
                // parse compound block of kvpairs
                KeyValueEntry::new(
                    Key::TopLevelObject(
                        CadmusObject::Table 
                            { 
                                id: 1,
                                name: String::from("Quotes_OCC") 
                            }
                        )
                    , token.source_loc
                    , BlockValue::Empty
                )
            }
            TokenValue::TableOccurrence => {
                KeyValueEntry::new(
                    Key::TopLevelObject(
                        CadmusObject::TableOccurrence 
                            { 
                                id: 1,
                                name: String::from("Quotes_OCC") 
                            }
                        )
                    , token.source_loc
                    , BlockValue::Empty
                )
            }
            _ => unreachable!()
        };

        entries.add(tmp);



    }

    Ok(KeyValueBlock::new())

}
