use crate::{cadlang::lexer::lex, schema::Schema};

use super::parser::{parse, validate_table_occurrence_references, validate_layout_references, validate_relation_references, ParseErr};

pub fn compile_to_schema(code: String) -> Result<Schema, ParseErr> {
    let tokens = lex(&code).expect("Unable to lex cadmus code.");
    let mut result = parse(&tokens)?;
    validate_table_occurrence_references(&mut result).expect("Unable to validate");
    validate_layout_references(&mut result).expect("Unable to validate");
    validate_relation_references(&mut result).expect("Unable to validate");
    Ok(result)
}
