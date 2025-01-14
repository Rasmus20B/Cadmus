
use super::record::Record;

#[derive(Debug, Clone)]
pub struct Table {
    pub id: usize,
    pub name: String,
    pub records: Vec<Record>,
    pub field_names: Vec<String>,
}

impl Table {
    pub fn new(id_: usize) -> Self {
        Self {
            id: id_,
            name: String::new(),
            records: vec![],
            field_names: vec![],
        }
    }

    pub fn name(mut self, name: &str) -> Self {
       self.name = name.to_string();
       self
    }

    pub fn new_record(&mut self) {
        let tmp = Record::new(self.records.len(), &self.field_names);
    }
}
