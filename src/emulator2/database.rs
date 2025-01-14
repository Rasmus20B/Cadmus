
use super::{
    table::Table,
    table_occurrence::TableOccurrence,
};

use std::rc::Rc;

use crate::schema::Schema;

pub struct Database {
    pub tables: Vec<Table>,
    pub table_occurrences: Vec<TableOccurrence>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            tables: vec![],
            table_occurrences: vec![],
        }
    }

    pub fn from_schema(schema: &Schema) -> Self {
        let mut tmp_db = Database::new();
        for (i, table) in &schema.tables {
            let mut tmp_table = Table::new(*i)
                .name(&table.name);
            tmp_table.field_names = table.fields
                .iter()
                .map(|field| field.1.name.clone())
                .collect();
            tmp_db.tables.push(tmp_table)
        }

        for (i, occurrence) in &schema.table_occurrences {
            let table = Rc::new(tmp_db.tables.iter()
                .find(|t| t.id == occurrence.base_table.top_id as usize)
                .inspect(|t| println!("{}", t.name))
                .unwrap().clone());

            println!("OCC name: {}", occurrence.name);


            tmp_db.table_occurrences.push(TableOccurrence::new(occurrence.id, occurrence.name.clone(), String::from("PLACEHOLDER"), table));
        }
        tmp_db
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use super::*;
    use crate::cadlang::compiler::compile_to_schema;
    #[test]
    fn from_cad_file() {
        let code = read_to_string("test_data/cad_files/initial.cad").unwrap();
        let cad = compile_to_schema(code).unwrap();

        let db = Database::from_schema(&cad);

        // Tables
        assert_eq!(db.tables.len(), 3);
        assert_eq!(db.tables.iter().filter(|t| (t.name == "Job")).count(), 1);
        assert_eq!(db.tables.iter().filter(|t| (t.name == "Person")).count(), 1);
        assert_eq!(db.tables.iter().filter(|t| (t.name == "SalaryLevel")).count(), 1);

        // Table Occurrences
        assert_eq!(db.table_occurrences.len(), 3);
        assert_eq!(db.table_occurrences.iter().filter(|t| (t.name == "Job_occ")).count(), 1);
        assert_eq!(db.table_occurrences.iter().filter(|t| (t.name == "Person_occ")).count(), 1);
        assert_eq!(db.table_occurrences.iter().filter(|t| (t.name == "Salary_occ")).count(), 1);
    }
}





