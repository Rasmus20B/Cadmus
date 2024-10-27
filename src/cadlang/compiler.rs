use crate::{cadlang::lexer::lex, schema::Schema};

use super::{parser::{parse, ParseErr}, validate::*};

pub fn compile_to_schema(code: String) -> Result<Schema, ParseErr> {
    let tokens = lex(&code).expect("Unable to lex cadmus code.");
    let (mut schema, bindings) = match parse(&tokens) {
        Ok((s, b)) => (s, b),
        Err(e) => { eprintln!("{:?}", e); panic!(); }
    };
    match validate(&mut schema, &bindings) {
        Ok(()) => {},
        Err(e) => { eprintln!("{:?}", e); panic!(); }
    };
    Ok(schema)
}
