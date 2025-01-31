
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use super::field::Field;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Table {
    pub id: usize,
    pub name: String,
    pub comment: String,
    pub fields: HashMap<usize, Field>,
    pub created_by: String,
    pub modified_by: String,
}

impl Table {
    pub fn new(id_: usize) -> Self {
        Self {
            id: id_,
            name: String::new(),
            comment: String::new(),
            fields: HashMap::new(),
            created_by: String::new(),
            modified_by: String::new(),
        }
    }
}
