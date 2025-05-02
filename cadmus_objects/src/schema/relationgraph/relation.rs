
use serde::{Serialize, Deserialize};

use crate::reference::FieldReference;

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

impl core::fmt::Display for RelationComparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationComparison::Equal => write!(f, "=="),
            RelationComparison::NotEqual => write!(f, "!="),
            RelationComparison::Less => write!(f, "<"),
            RelationComparison::LessEqual => write!(f, "<="),
            RelationComparison::Greater => write!(f, ">"),
            RelationComparison::GreaterEqual => write!(f, ">="),
            RelationComparison::Cartesian => write!(f, "*"),
        }
    }
}

impl RelationComparison {
    pub fn mirrored(&self) -> Self {
        match self {
            Self::Equal => Self::Equal,
            Self::NotEqual => Self::NotEqual,
            Self::Greater => Self::LessEqual,
            Self::GreaterEqual => Self::Less,
            Self::LessEqual => Self::Greater,
            Self::Less => Self::GreaterEqual,
            Self::Cartesian => Self::Cartesian,
        }
    }
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
