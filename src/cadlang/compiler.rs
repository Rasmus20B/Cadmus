use crate::cadlang::lexer::lex;

use crate::dbobjects::schema::Schema;

use std::path::Path;
use std::fs::read_to_string;

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

pub fn compile_to_file(path: &Path) -> Result<File, CompileErr> {
    let working_dir = path.parent().unwrap();

    println!("working directory: {:?}", working_dir);

    println!("PATH: {:?}", path);
    let code = read_to_string(path).unwrap();
    let tokens = lex(&code).expect("Unable to lex cadmus code.");
    let staging = match parse(&tokens) {
        Ok(s) => s,
        Err(e) => { eprintln!("{:?}", e); panic!(); }
    };

    let file = build_file(&staging, working_dir);

    Ok(file)
}
