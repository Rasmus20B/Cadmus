
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

pub struct RelationCriteria {
    field_self: u32,
    field_other: u32,
    comparison: RelationComparison,
}

pub struct Relation {
    table_occurrence_id: u32,
    criteria: Vec<RelationCriteria>,
}
