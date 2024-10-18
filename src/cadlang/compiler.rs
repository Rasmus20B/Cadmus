use crate::{cadlang::lexer::lex, schema::Schema};

use super::parser::parse;

pub fn compile_to_schema(code: String) -> Schema {
    let tokens = lex(&code).expect("Unable to lex cadmus code.");
    parse(&tokens).expect("Unable to parse tokens.")
}
