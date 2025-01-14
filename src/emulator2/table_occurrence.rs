
use std::rc::Rc;
use super::{table::Table, record::Record};

pub struct TableOccurrence {
    id: usize,
    pub name: String,
    data_source: String,
    table: Rc<Table>
}

impl TableOccurrence {
    pub fn new(id_: usize, name_: String, data_source_: String, table_: Rc<Table>) -> Self {
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
