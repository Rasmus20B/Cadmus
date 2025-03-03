
pub mod field;
pub mod table;
pub mod relationgraph;

use super::schema::relationgraph::graph::RelationGraph;
use super::schema::table::Table;

#[derive(Debug, PartialEq, Eq)]
pub struct Schema {
    pub tables: Vec<Table>,
    pub relation_graph: RelationGraph,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            tables: vec![],
            relation_graph: RelationGraph::new(),
        }
    }
}
