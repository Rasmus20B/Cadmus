
use super::record::Record;
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct RecordStore {
    pub storage: RefCell<Vec<Record>>,
}

impl RecordStore {
    pub fn new() -> Self {
        Self {
            storage: RefCell::new(vec![])
        }
    }
}
