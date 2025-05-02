
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, HashMap};

use crate::file::File;

use super::field::Field;

pub type TableID = u32;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Table {
    pub id: u32,
    pub name: String,
    pub comment: String,
    pub fields: BTreeMap<u32, Field>,
    pub created_by: String,
    pub modified_by: String,
}

impl Table {
    pub fn new(id_: u32) -> Self {
        Self {
            id: id_,
            name: String::new(),
            comment: String::new(),
            fields: BTreeMap::new(),
            created_by: String::new(),
            modified_by: String::new(),
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn to_cad(&self, file: &File, externs: &HashMap<usize, File>) -> String {
        let mut buffer = String::new();
        buffer.push_str(&format!("table %{} {} = {{\n", self.id, self.name));
        for (_, field) in &self.fields {
            buffer.push_str(&format!("    {}\n", &field.to_cad(file, externs)));
        }

        buffer.push_str("}");
        buffer
    }
}
