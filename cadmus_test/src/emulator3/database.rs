
use crate::dbobjects::file::File;
use super::record_store::RecordStore;

pub struct Database {
    pub file: File,
    pub records: RecordStore,
}

impl Database {
    pub fn from_file(file_: File) -> Self {
        Self {
            records: RecordStore::new(&file_.schema.tables),
            file: file_,
        }
    }
}
