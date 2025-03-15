
use std::collections::HashMap;
use std::path::Path;
use crate::{cadlang, hbam2};

use super::schema::Schema;
use super::scripting::script::Script;
use super::layout::Layout;
use super::data_source::*;

#[derive(Debug, PartialEq, Clone)]
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
        let externs = &self.data_sources.iter().map(|ds| (ds.id as usize, 
                match ds.dstype {
                    DataSourceType::FileMaker => {
                        let mut ctx = hbam2::Context::new();
                        ctx.get_schema_contents(&(self.working_dir.clone() + "/" + &ds.paths[0] + ".fmp12"))
                    },
                    DataSourceType::Cadmus => {
                        cadlang::compiler::compile_to_file(
                            Path::new(&(self.working_dir.clone() + "/" + &ds.paths[0])))
                            .unwrap()
                    }
                    _ => {
                        todo!()
                    },
        })).collect::<HashMap<usize, File>>();
        buffer.push_str("\n\n");
        buffer.push_str(&self.schema.to_cad(self, externs));
        buffer.push_str("\n\n");
        buffer.push_str(&self.layouts.iter()
            .map(|layout| layout.to_cad(self))
            .collect::<Vec<String>>().join(&"\n"));
        buffer
    }

    pub fn to_cad_with_externs(&self, externs: &Vec<File>) -> String {
        let mut buffer = String::new();

        for file in externs {
            println!("file: {}", file.name);
        }
        
        let extern_map = self.data_sources
            .iter()
            .map(|ds| (ds.id, externs
                    .iter()
                    .inspect(|e| println!("{} == {}?", (self.working_dir.clone() + "/" + &ds.name + ".fmp12"), e.name))
                    .find(|e| ds.paths
                        .iter()
                        .find(|p| **p == Path::new(&e.name).to_path_buf().file_stem().unwrap().to_string_lossy()).is_some())
            ))
            .filter(|e| e.1.is_some())
            .map(|e| (e.0 as usize, e.1.unwrap().clone()))
            .collect::<HashMap<usize, File>>();

        buffer.push_str(&self.schema.to_cad(self, &extern_map));
        buffer.push_str("\n\n");
        buffer.push_str(&self.layouts.iter()
            .map(|layout| layout.to_cad(self))
            .collect::<Vec<String>>().join(&"\n"));
        buffer.push_str("\n\n");

        buffer.push_str(&self.scripts.iter()
            .map(|layout| layout.to_cad(self, extern_map.clone()))
            .collect::<Vec<String>>().join(&"\n"));
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
