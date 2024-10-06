use crate::compile::parser;
use crate::compile::lexer;
use crate::schema::Schema;

pub fn compile_burn(code: &str) -> Schema {
    let tokens = lexer::tokenize(&code);
    let p = parser::Parser::new(tokens);
    p.parse_program().expect("unable to parse program. ")
}


