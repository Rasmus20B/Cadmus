use serde::{Serialize, Deserialize};

use crate::emulator2::database_mgr::DatabaseMgr;

pub enum ResolveErr {
}

pub trait Resolvable {
    type Output;
    fn resolve(&self, db_mgr: &DatabaseMgr) -> Result<Self::Output, ResolveErr>;
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct FieldReference {
    data_source: u32,
    table_occurrence_id: u32,
    field_id: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TableReference {
    data_source: u32,
    table_id: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ScriptReference {
    data_source: u32,
    script_id: u32,
}
