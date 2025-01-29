
use serde::{Serialize, Deserialize};

use crate::dbobjects::reference::FieldReference;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum RelationComparison {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Cartesian
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct RelationCriteria {
    pub field_self: u32,
    pub field_other: u32,
    pub comparison: RelationComparison,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Relation {
    pub id: u32,
    pub other_occurrence: u32,
    pub criteria: Vec<RelationCriteria>,
}
