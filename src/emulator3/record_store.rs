
use std::collections::HashMap;
use std::cmp::Ordering;

use crate::dbobjects::schema::table::{Table, TableID};

#[derive(Debug, PartialEq)]
pub struct Record {
    pub id: u32,
    pub fields: Vec<(u32, String)>,
}

#[derive(Debug, PartialEq)]
pub struct RecordStore {
    pub records_by_table: HashMap<TableID, Vec<Record>>,
}

impl RecordStore {
    pub fn new(tables: &Vec<Table>) -> Self {
        Self {
            records_by_table: tables.iter().map(|table| (table.id, vec![])).collect(),
        }
    }

    pub fn new_record(&mut self, table: &Table) -> u32 {
        let records = self.records_by_table.get_mut(&table.id).unwrap();

        records.push(Record {
            id: records.iter().max_by(|a, b| a.id.cmp(&b.id)).unwrap_or(&Record { id: 0, fields: vec![] }).id + 1,
            fields: table.fields.iter().inspect(|f| println!("pushing field: {}", f.1.name)).map(|field| (field.1.id, String::new())).collect(),
        });

        records.iter().max_by(|a, b| a.id.cmp(&b.id)).unwrap().id
    }

    pub fn set_field(&mut self, table: u32, record: u32, field: u32, value: String) {
        let records = self.records_by_table.get_mut(&table).unwrap();
        let record = &mut records.iter_mut().find(|search| search.id == record).unwrap();
        let field = record.fields.iter_mut().find(|search| search.0 == field).unwrap();

        field.1 = value;
    }

    pub fn get_field(&self, table: u32, record: u32, field: u32) -> String {
        let records = self.records_by_table.get(&table).unwrap();
        let record = &mut records.iter().find(|search| search.id == record).unwrap();
        let field = record.fields.iter().find(|search| search.0 == field).unwrap();
        field.1.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{RecordStore, Table};
    use crate::dbobjects::schema::field::*;
    use std::collections::BTreeMap;
    #[test]
    fn records_test() {
        let mut tables = vec![];
        let mut table = Table {
            id: 1,
            name: "Person".to_string(),
            created_by: String::new(),
            modified_by: String::new(),
            comment: String::new(),
            fields: BTreeMap::new(),
        };

        table.fields.insert(1, Field::new(1, "id".to_string()));
        table.fields.insert(2, Field::new(2, "first_name".to_string()));
        table.fields.insert(3, Field::new(3, "surname".to_string()));

        tables.push(table);

        let mut records = RecordStore::new(&tables);
        records.new_record(&tables[0]);

        println!("{:?}", records.records_by_table.get(&1).unwrap()[0]);

        assert_eq!(records.records_by_table.len(), 1);
        assert_eq!(records.records_by_table.get(&1).unwrap().len(), 1);
        records.set_field(1, 1, 1, "1".to_string());
        records.set_field(1, 1, 2, "John".to_string());
        records.set_field(1, 1, 3, "Smith".to_string());
        assert_eq!(records.get_field(1, 1, 1), "1");
        assert_eq!(records.get_field(1, 1, 2), "John");
        assert_eq!(records.get_field(1, 1, 3), "Smith");
        records.new_record(&tables[0]);
        records.set_field(1, 2, 1, "2".to_string());
        records.set_field(1, 2, 2, "Jeff".to_string());
        records.set_field(1, 2, 3, "Keighly".to_string());
        assert_eq!(records.get_field(1, 2, 1), "2");
        assert_eq!(records.get_field(1, 2, 2), "Jeff");
        assert_eq!(records.get_field(1, 2, 3), "Keighly");
        assert_eq!(records.records_by_table.get(&1).unwrap().len(), 2);
        assert_eq!(records.records_by_table.get(&1).unwrap()[0].fields.len(), 3);
        assert_eq!(records.records_by_table.get(&1).unwrap()[1].fields.len(), 3);
        assert_eq!(records.records_by_table.get(&1).unwrap().get(2), None);
        records.set_field(1, 1, 2, "Jeff".to_string());
        assert_eq!(records.get_field(1, 1, 2), "Jeff");
    }
}
