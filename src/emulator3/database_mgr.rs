
use std::collections::HashMap;

use crate::dbobjects::reference::{FieldReference, TableOccurrenceReference};

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

    /* We need to supply information about the record, and record ID itself.
     * This includes: Table occurrence and record ID.
     * We also need to supply information about the field we are looking for:
     * This is just the FieldReference itself. */
    pub fn get_field(&self,
        from: (TableOccurrenceReference, u32),
        to: FieldReference,
        active_database: &str
        ) -> String {

        let cur_db = self.databases.get(active_database).unwrap();
        let graph = &cur_db.file.schema.relation_graph;
        let target_database = cur_db.file.data_sources.iter()
            .find(|source| source.id == to.data_source);

        let from_table_id = cur_db
            .file.schema.relation_graph.nodes.iter()
            .find(|to| to.id == from.0.table_occurrence_id).unwrap()
            .base.table_id;

        
            todo!();


    }

    pub fn load_file(&mut self, path: &str) -> &Database {
        println!("opening {}", path);
        if path.ends_with(".cad") {
            // load cad file
            let cadcode = read_to_string(&path).unwrap();
            let file = crate::cadlang::compiler::compile_to_file(cadcode).unwrap();
            let database = Database::from_file(file);
            self.databases.insert(path.to_string(), database);
            self.databases.get(path).unwrap()
        } else if path.ends_with(".fmp12") {
            // load hbam file
            todo!()
        } else {
            todo!()
        }
    }
}
