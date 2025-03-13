
use std::collections::HashMap;
use std::path::Path;
use crate::cadlang;

use super::schema::Schema;
use super::scripting::script::Script;
use super::layout::Layout;
use super::data_source::*;

#[derive(Debug, PartialEq)]
pub struct File {
    pub name: String,
    pub schema: Schema,
    pub layouts: Vec<Layout>,
    pub data_sources: Vec<DataSource>,
    pub scripts: Vec<Script>,
    pub tests: Vec<Script>,
    pub working_dir: String,
}

impl File {
    pub fn to_cad(&self) -> String {
        let mut buffer = String::new();

        buffer.push_str(&self.data_sources.iter().map(|ds| ds.to_cad()).collect::<Vec<String>>().join(&"\n"));
        let externs = &self.data_sources.iter().map(|ds| (ds.id as usize, cadlang::compiler::compile_to_file(Path::new(&(self.working_dir.clone() + "/" + &ds.paths[0]))).unwrap())).collect::<HashMap::<usize, File>>();
        buffer.push_str("\n\n");
        buffer.push_str(&self.schema.to_cad(self, externs));
        buffer.push_str("\n\n");

        buffer
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::cadlang;

    use super::*;

    #[test]
    fn quotes_file_test() {
        let code = std::fs::read_to_string(Path::new("test_data/cad_files/multi_file_solution/quotes.cad")).unwrap();
        let file = cadlang::compiler::compile_to_file(Path::new("test_data/cad_files/multi_file_solution/quotes.cad")).unwrap();

        println!("{}", file.to_cad());
    }
}
