
use std::collections::HashMap;
use super::{table_occurrence::TableOccurrence, relation::Relation};

pub struct RelationGraph {
    nodes: HashMap<TableOccurrence, Vec<Relation>>,
}
