use crate::{cadlang::lexer::lex, schema::Schema};

use super::parser::{parse, ParseErr};

pub fn compile_to_schema(code: String) -> Result<Schema, ParseErr> {
    let tokens = lex(&code).expect("Unable to lex cadmus code.");
    parse(&tokens)
}
