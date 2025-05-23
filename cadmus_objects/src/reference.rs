use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct FieldReference {
    pub data_source: u32,
    pub table_occurrence_id: u32,
    pub field_id: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TableOccurrenceReference {
    pub data_source: u32,
    pub table_occurrence_id: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TableReference {
    pub data_source: u32,
    pub table_id: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ScriptReference {
    pub data_source: u32,
    pub script_id: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LayoutReference {
    pub file_name: String,
    pub layout_id: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ValueListReference {
    pub data_source: u32,
    pub value_list_id: u32,
}

