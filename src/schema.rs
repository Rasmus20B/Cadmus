use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum DBObjectKind {
    Table,
    TableOccurrence,
    Field,
    Relation,
    ValueList,
    Script,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct DBObject {
    pub id: usize,
    pub name: String,
    pub kind: DBObjectKind,
}

pub struct Schema {
    pub objects: Vec<DBObject>,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            objects: vec![],
        }
    }
}

