
pub mod field;
pub mod table;
pub mod relationgraph;

use std::collections::HashMap;

use super::file::File;
use super::schema::relationgraph::graph::RelationGraph;
use super::schema::table::Table;

#[derive(Debug, PartialEq, Eq, Clone)]
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

    pub fn to_cad(&self, file: &File, externs: &HashMap<usize, File>) -> String {
        let mut buffer = String::new();
        buffer.push_str(&self.tables.iter().map(|table| table.to_cad(file, externs)).collect::<Vec<String>>().join(&"\n"));
        buffer.push_str("\n\n");
        buffer.push_str(&self.relation_graph.to_cad(file, externs));
        buffer
    }
}

