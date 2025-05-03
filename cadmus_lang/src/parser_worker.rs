use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use crate::{
    diagnostic::Diagnostic,
    error::{Error, Result},
    lexer::lex,
    parsed_file::ParsedFile,
    proto_schema::ProtoSchema,
};

pub struct ParserWorker {
    path: PathBuf,
    contents: String,
}

impl ParserWorker {
    pub fn new() -> Self {
        Self {
            path: PathBuf::new(),
            contents: String::new(),
        }
    }

    pub fn with_path(&mut self, path: PathBuf) -> &mut Self {
        self.path = path;
        self
    }

    pub fn build(&mut self) -> Result<ParsedFile> {
        let result = ParsedFile::new();
        self.contents = read_to_string(&self.path).map_err(Error::Fs)?;

        let mut diagnostics = vec![];

        let tokens = lex(&self.contents, &mut diagnostics)?;

        Ok(result)
    }
}
