
use super::database::Database;
use std::collections::HashMap;
use std::path::Path;
use std::fs::read_to_string;

use super::{data_source::*, record::*};
use crate::{hbam2, cadlang};

pub struct DatabaseMgr {
    pub databases: HashMap<String, Database>
}

impl DatabaseMgr {

    pub fn new() -> Self {
        Self {
            databases: HashMap::new()
        }
    }

    pub fn load_cadmus_file(&mut self, path: &Path) -> Option<&Database> {
        let filename = path.to_str().unwrap().to_string();
        let code = read_to_string(path).unwrap();
        let schema = cadlang::compiler::compile_to_schema(code).expect("Unable to compile cadmus file.");
        self.databases.insert(filename.clone(), Database::from_schema(&schema));
        Some(self.databases.get(&filename).unwrap())
    }

    pub fn add_record(&mut self, file: String, table_id: usize) {
        let db = self.databases.get_mut(&file).unwrap();
        db.tables.iter_mut().find(|t| t.id == table_id).unwrap()
            .new_record()
    }

    pub fn load_data_source(&mut self, source: &DataSource) {
        let tmp_db = match source.dstype {
            DataSourceType::FileMaker => {
                Database::new()
            },
            DataSourceType::Cadmus => {
                Database::new()
            },
            DataSourceType::ODBC => todo!("ODBC decoding has not been implemented."),
        };

        self.databases.insert(source.filename.clone(), tmp_db);
    }
}
