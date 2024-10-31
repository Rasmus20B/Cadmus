use crate::{cadlang::lexer::lex, schema::Schema};

use super::{parser::parse, error::CompileErr, staging::Stage}; 

pub fn compile_to_schema(code: String) -> Result<Schema, CompileErr> {
    let tokens = lex(&code).expect("Unable to lex cadmus code.");
    let staging = match parse(&tokens) {
        Ok(s) => s,
        Err(e) => { eprintln!("{:?}", e); panic!(); }
    };

    match staging.to_schema() {
        Ok(schema) => {
            return Ok(schema)
        },
        Err(errs) => {
            for e in &errs {
                eprintln!("error: {:?}", e);
            }
        }
    }
    return Ok(Schema::new());
}
