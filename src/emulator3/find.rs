

use crate::dbobjects::{file::File, reference::{TableOccurrenceReference, FieldReference}};

use super::database::Database;

pub enum RequestType {
    Find,
    Omit
}

pub struct FindRequest {
    fields: Vec<FieldReference>,
    criteria: Vec<String>,
}

pub struct Find {
    requests: Vec<FindRequest>,
}

impl Find {
    pub fn new() -> Self {
        Self {
            requests: vec![],
        }
    }
}

pub struct FoundSet {
    pub table_occurrence_ref: TableOccurrenceReference,
    pub records: Vec<u32>,
    pub current_find: Find,
    pub cursor: Option<u32>,
}

impl FoundSet {
    pub fn new(occurrence: u32, database: &Database) -> Self {
        let table_id = database.file.schema.relation_graph.nodes.iter().find(|node| node.id == occurrence).unwrap().base.table_id;
        let records_ = database.records.records_by_table
            .get(&table_id)
            .map(|record_list| record_list.iter().map(|record| record.id).collect())
            .unwrap_or(vec![]);

        Self {
            table_occurrence_ref: TableOccurrenceReference {
                data_source: 0,
                table_occurrence_id: occurrence,
            },
            current_find: Find::new(),
            cursor: if records_.is_empty() { None } else { Some(0) },
            records: records_,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use std::path::Path;
    use super::{Database, super::window::Window};
    #[test]
    fn basic_find() {
        let db_file = crate::cadlang::compiler::compile_to_file(read_to_string(Path::new("test_data/cad_files/multi_file_solution/customers.cad")).unwrap()).unwrap();
        let mut database = Database::from_file(db_file);
        let mut window = Window::new()
            .name(database.file.name.clone())
            .database(database.file.name.clone());
        window.init_found_sets(&database);
        assert_eq!(window.found_sets
            .iter()
            .find(|set| set.table_occurrence_ref.table_occurrence_id == 1)
            .unwrap().records.len(), 0);
        let new_id = database.records.new_record(&database.file.schema.tables[0]);
        window.append_record_to_found_sets(new_id, database.file.schema.tables[0].id, &database);

        // First 2 found sets should be updated, as the occurrences they use rely on the same table
        assert_eq!(window.found_sets
            .iter()
            .find(|set| set.table_occurrence_ref.table_occurrence_id == 1)
            .unwrap().records.len(), 1);

        assert_eq!(window.found_sets
            .iter()
            .find(|set| set.table_occurrence_ref.table_occurrence_id == 2)
            .unwrap().records.len(), 1);

        // Last found set should not be updated, as the occurrence uses a different table
        assert_eq!(window.found_sets
            .iter()
            .find(|set| set.table_occurrence_ref.table_occurrence_id == 3)
            .unwrap().records.len(), 0);
    }
}




