use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum DBObjectKind {
    Table,
    TableOccurrence,
    Field,
    Relation,
    ValueList,
    Script,
    Layout,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum DBObjectStatus {
    Unmodified,
    Modified,
    Created,
    Deleted,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Table {
    pub id: usize,
    pub name: String,
    pub created_by: String,
    pub modified_by: String,
}

impl Table {
    pub fn new(id_: usize) -> Self {
        Self {
            id: id_,
            name: String::new(),
            created_by: String::new(),
            modified_by: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Field {
    id: usize,
    name: String,
    created_by: String,
    modified_by: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TableOccurrence {
    pub id: usize,
    pub name: String,
    pub created_by: String,
    pub modified_by: String,
}

impl TableOccurrence {
    pub fn new(id_: usize) -> Self {
        Self {
            id: id_,
            name: String::new(),
            created_by: String::new(),
            modified_by: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum RelationComparison {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Cartesian
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Relation {
    pub id: usize,
    pub table1: u16,
    pub table1_name: String,
    pub table1_data_source: u8,
    pub field1: u16,
    pub table2: u16,
    pub table2_name: String,
    pub table2_data_source: u8,
    pub field2: u16,
    pub comparison: RelationComparison,
}

impl Relation {
    pub fn new(id_: usize) -> Self {
        Self {
            id: id_,
            table1: 0,
            table1_name: String::new(),
            table1_data_source: 0,
            field1: 0,
            table2: 0,
            table2_name: String::new(),
            table2_data_source: 0,
            field2: 0,
            comparison: RelationComparison::Equal,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ValueLists {
    id: usize,
    name: String,
    created_by: String,
    modified_by: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Script {
    id: usize,
    name: String,
    created_by: String,
    modified_by: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Test {
    id: usize,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Schema {
    pub tables: HashMap<usize, Table>,
    pub fields: HashMap<usize, Field>,
    pub table_occurrences: HashMap<usize, TableOccurrence>,
    pub relations: HashMap<usize, Relation>,
    pub value_lists: HashMap<usize, ValueLists>,
    pub scripts: HashMap<usize, Script>,
    pub tests: HashMap<usize, Test>,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            fields: HashMap::new(),
            table_occurrences: HashMap::new(),
            relations: HashMap::new(),
            value_lists: HashMap::new(),
            scripts: HashMap::new(),
            tests: HashMap::new(),
        }
    }
}
