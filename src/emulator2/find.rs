
use std::rc::Rc;
use super::{table_occurrence::TableOccurrence, record::Record};

pub struct FindRequest {
    table_occurrence: String,
    field: String,
    expression: String,
}

pub struct FoundSet {
    record_ptr: usize,
    records: Vec<usize>,
    find: Vec<FindRequest>,
    occurrence: Rc<TableOccurrence>,
}

impl FoundSet {

    pub fn current_record(&self) -> Option<&Record> {
        if self.records.is_empty() {
            return None
        }
        Some(&self.occurrence.get_records()[self.record_ptr])
    }
}

