
use std::collections::HashMap;
use super::{table_occurrence::TableOccurrence, relation::Relation};

pub struct RelationGraph {
    pub nodes: Vec<TableOccurrence>
}

impl RelationGraph {
    pub fn new() -> Self {
        Self {
            nodes: vec![]
        }
    }
}
