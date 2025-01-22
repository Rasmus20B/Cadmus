
pub mod field;
pub mod table;
pub mod relationgraph;

use super::schema::relationgraph::graph::RelationGraph;
use super::schema::table::Table;

pub struct Schema {
    tables: Vec<Table>,
    relation_graph: RelationGraph,

}
