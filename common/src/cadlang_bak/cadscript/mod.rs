
pub(crate) mod lexer;
pub(crate) mod parser;
pub(crate) mod token;
pub(crate) mod proto_script;
pub(crate) mod proto_instruction;
mod arg_lookups;

use proto_script::*;

pub fn compile_cadscript(code: &str) -> ProtoScript {
    let tokens = lexer::lex(code);
    let script = parser::parse(tokens).unwrap();

    ProtoScript {
        name: String::new(),
        instructions: script,
    }
}
