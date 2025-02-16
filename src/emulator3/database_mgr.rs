
use std::collections::HashMap;

use super::database::Database;
use std::fs::read_to_string;

pub struct DatabaseMgr {
    pub databases: HashMap::<String, Database>,
}

impl DatabaseMgr {
    pub fn new() -> Self {
        Self {
            databases: HashMap::new(),
        }
    }

    pub fn load_file(&mut self, path: String) -> &Database {
        if path.ends_with(".cad") {
            // load cad file
            let cadcode = read_to_string(&path).unwrap();
            let file = crate::cadlang::compiler::compile_to_file(cadcode).unwrap();
            let database = Database::from_file(file);
            self.databases.insert(path.clone(), database);
            self.databases.get(&path).unwrap()
        } else if path.ends_with(".fmp12") {
            // load hbam file
            todo!()
        } else {
            todo!()
        }
    }
}
