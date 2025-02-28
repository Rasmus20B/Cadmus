use crate::cadlang::lexer::lex;

use crate::dbobjects::schema::Schema;

use super::{parser::parse, error::CompileErr}; 
use super::backend::build_file;
use crate::dbobjects::file::File;

#[deprecated]
pub fn compile_to_schema(code: String) -> Result<Schema, CompileErr> {
    let tokens = lex(&code).expect("Unable to lex cadmus code.");
    let staging = match parse(&tokens) {
        Ok(s) => s,
        Err(e) => { eprintln!("{:?}", e); panic!(); }
    };

    //match staging.to_schema() {
    //    Ok(schema) => {
    //        return Ok(schema)
    //    },
    //    Err(errs) => {
    //        for e in &errs {
    //            eprintln!("error: {:?}", e);
    //        }
    //    }
    //}
    Ok(Schema::new())
}

pub fn compile_to_file(code: String) -> Result<File, CompileErr> {
    let tokens = lex(&code).expect("Unable to lex cadmus code.");
    let staging = match parse(&tokens) {
        Ok(s) => s,
        Err(e) => { eprintln!("{:?}", e); panic!(); }
    };

    let file = build_file(&staging);

    Ok(file)
}
