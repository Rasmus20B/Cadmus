
use super::{database::Database, find::*};

use crate::dbobjects::schema::table::{Table, TableID};

pub struct Window {
    pub id: u32,
    pub name: String,
    pub database: String,
    pub layout_id: u32,
    pub found_sets: Vec<FoundSet>,
}

impl Window {
    pub fn new() -> Self {
        Self {
            id: 0,
            name: String::new(),
            database: String::new(),
            layout_id: 0,
            found_sets: vec![],
        }
    }

    pub fn id(mut self, id: u32) -> Self {
        self.id = id;
        self
    }

    pub fn name(mut self, name_: String) -> Self {
        self.name = name_;
        self
    }

    pub fn database(mut self, database_: String) -> Self {
        self.database = database_;
        self
    }

    pub fn layout_id(mut self, layout_id: u32) -> Self {
        self.layout_id = layout_id;
        self
    }

    pub fn init_found_sets(&mut self, database: &Database) {
        for occurrence in &database.file.schema.relation_graph.nodes {
            self.found_sets.push(FoundSet::new(occurrence.id, database));
        }
    }

    pub fn append_record_to_found_sets(&mut self, record_id: u32, table_id: TableID, database: &Database) {
        let effected_occurrences = database.file.schema.relation_graph
            .nodes
            .iter()
            .filter(|occ| occ.base.table_id == table_id)
            .map(|occ| occ.id);

        for occurrence in effected_occurrences {
            self.found_sets.iter_mut()
                .filter(|set| set.table_occurrence_ref.table_occurrence_id == occurrence)
                .for_each(|set| { set.records.push(record_id); set.cursor = Some(set.records.len() as u32 - 1)});
        }


    }
}
