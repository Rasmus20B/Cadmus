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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Field {
    id: usize,
    name: String,
    created_by: String,
    modified_by: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TableOccurrence {
    id: usize,
    name: String,
    created_by: String,
    modified_by: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Relation {
    id: usize,
    left: String,
    right: String,
    created_by: String,
    modified_by: String,
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
pub struct TrackedDBObject<T> {
    pub object: T,
    pub status: DBObjectStatus,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Schema {
    pub tables: Vec<Table>,
    pub fields: Vec<Field>,
    pub table_occurrences: Vec<TableOccurrence>,
    pub relations: Vec<Relation>,
    pub value_lists: Vec<ValueLists>,
    pub scripts: Vec<Script>,
    pub tests: Vec<Test>,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            tables: vec![],
            fields: vec![],
            table_occurrences: vec![],
            relations: vec![],
            value_lists: vec![],
            scripts: vec![],
            tests: vec![],
        }
    }
}
