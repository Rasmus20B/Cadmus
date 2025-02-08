use serde::{Serialize, Deserialize};

use crate::emulator2::database_mgr::DatabaseMgr;

pub enum ResolveErr {
}

pub trait Resolvable {
    type Output;
    fn resolve(&self, from: String, db_mgr: &DatabaseMgr) -> Result<Self::Output, ResolveErr>;
}

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
    pub layout_id: u32,
}
