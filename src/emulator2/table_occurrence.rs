
use std::rc::Rc;
use std::cell::RefCell;
use super::{table::Table, record_store::RecordStore, record::Record, data_source::DataSource};

pub struct TableOccurrence {
    pub id: usize,
    pub name: String,
    pub data_source: usize,
    pub table: Rc<Table>
}

impl TableOccurrence {
    pub fn new(id_: usize, name_: String, data_source_: usize, table_: Rc<Table>) -> Self {
        Self {
            id: id_,
            name: name_,
            data_source: data_source_,
            table: table_
        }
    }
    pub fn get_records(&self) -> &Vec<Record> {
        &self.table.records
    }
}
