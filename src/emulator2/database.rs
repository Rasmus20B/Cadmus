
use super::{
    table::Table,
    table_occurrence::TableOccurrence,
    data_source::{DataSource, DataSourceType},
    layout::Layout,
};
use std::rc::Rc;
use crate::schema::Schema;

use crate::dbobjects::{file::File, scripting::script::Script};

pub struct Database {
    pub filename: String,
    pub name: String,
    pub tables: Vec<Table>,
    pub table_occurrences: Vec<TableOccurrence>,
    pub layouts: Vec<Rc<Layout>>,
    pub scripts: Vec<Script>,
    pub external_sources: Vec<DataSource>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            filename: String::new(),
            name: String::new(),
            tables: vec![],
            table_occurrences: vec![],
            layouts: vec![],
            scripts: vec![],
            external_sources: vec![],
        }
    }

    pub fn from_schema(file: &File) -> Self {
        let mut tmp_db = Database::new();
        let schema = &file.schema;
        for table in &schema.tables {
            let mut tmp_table = Table::new(table.id)
                .name(&table.name);
            tmp_table.field_names = table.fields
                .iter()
                .map(|field| field.1.name.clone())
                .collect();
            tmp_db.tables.push(tmp_table);
        }

        //for (i, source) in &schema.data_sources {
        //    let tmp = DataSource {
        //        id: source.id,
        //        dstype: DataSourceType::FileMaker,
        //        name: source.name.clone(),
        //        filename: source.filename.clone(),
        //    };
        //}
        //
        //for (occurrence, relations) in &schema.relation_graph.nodes {
        //    let table = Rc::new(tmp_db.tables.iter()
        //        .find(|t| t.id == occurrence.base.table_id as usize)
        //        .unwrap().clone());
        //
        //    tmp_db.table_occurrences.push(TableOccurrence::new(
        //            occurrence.id,
        //            occurrence.name.clone(),
        //            occurrence.base_table.data_source as usize,
        //            table)
        //        );
        //}
        //
        //for (i, layout) in &schema.layouts {
        //    let tmp_layout = Layout::new(*i, layout.name.clone(), layout.table_occurrence.top_id as usize);
        //    tmp_db.layouts.push(tmp_layout.into());
        //}

        //tmp_db.scripts.extend(schema.scripts.iter().map(|x| x.1.clone()).collect::<Vec<Script>>());
        tmp_db
    }
}

//#[cfg(test)]
//mod tests {
//    use std::fs::read_to_string;
//    use super::*;
//    use crate::cadlang::compiler::compile_to_schema;
//    #[test]
//    fn from_cad_file() {
//        let code = read_to_string("test_data/cad_files/initial.cad").unwrap();
//        let cad = compile_to_schema(code).unwrap();
//
//        let db = Database::from_schema(&cad);
//
//        // Tables
//        assert_eq!(db.tables.len(), 3);
//        assert_eq!(db.tables.iter().filter(|t| (t.name == "Job")).count(), 1);
//        assert_eq!(db.tables.iter().filter(|t| (t.name == "Person")).count(), 1);
//        assert_eq!(db.tables.iter().filter(|t| (t.name == "SalaryLevel")).count(), 1);
//
//        // Table Occurrences
//        assert_eq!(db.table_occurrences.len(), 3);
//        assert_eq!(db.table_occurrences.iter().filter(|t| (t.name == "Job_occ")).count(), 1);
//        assert_eq!(db.table_occurrences.iter().filter(|t| (t.name == "Person_occ")).count(), 1);
//        assert_eq!(db.table_occurrences.iter().filter(|t| (t.name == "Salary_occ")).count(), 1);
//    }
//    #[test]
//    fn from_multi_cad_file() {
//        let code = read_to_string("test_data/cad_files/multi_file_solution/quotes.cad").unwrap();
//        let cad = compile_to_schema(code).unwrap();
//
//        let db = Database::from_schema(&cad);
//
//        // Tables
//        assert_eq!(db.tables.len(), 3);
//        assert_eq!(db.tables.iter().filter(|t| (t.name == "Job")).count(), 1);
//        assert_eq!(db.tables.iter().filter(|t| (t.name == "Person")).count(), 1);
//        assert_eq!(db.tables.iter().filter(|t| (t.name == "SalaryLevel")).count(), 1);
//
//        // Table Occurrences
//        assert_eq!(db.table_occurrences.len(), 3);
//        assert_eq!(db.table_occurrences.iter().filter(|t| (t.name == "Job_occ")).count(), 1);
//        assert_eq!(db.table_occurrences.iter().filter(|t| (t.name == "Person_occ")).count(), 1);
//        assert_eq!(db.table_occurrences.iter().filter(|t| (t.name == "Salary_occ")).count(), 1);
//    }
//}
//
//
//
//
//
