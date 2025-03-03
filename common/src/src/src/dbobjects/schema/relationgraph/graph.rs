
use std::collections::HashMap;
use super::{table_occurrence::TableOccurrence, relation::Relation};
use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq)]
pub struct RelationGraph {
    pub nodes: Vec<TableOccurrence>
}

impl RelationGraph {
    pub fn new() -> Self {
        Self {
            nodes: vec![]
        }
    }

    pub fn get_path(&self, node1: u32, node2: u32) -> Option<Vec<u32>> {
        let mut queue = VecDeque::new();
        let mut visited = vec![];
        let start = self.nodes.iter()
            .find(|node| node.id == node1)
            .unwrap();

        visited.push((node1, None));
        queue.push_back(node1);

        while !queue.is_empty() {
            let current_occurrence_id = queue.pop_front().unwrap();
            let current_occurrence = self.nodes.iter()
                .find(|node| node.id == current_occurrence_id)
                .unwrap();

            if current_occurrence_id == node2 {
                let mut path = vec![node2];
                let mut cur = node2;
                while let Some(parent) = visited.iter().find(|par| par.0 == cur) {
                    if let Some(par) = parent.1 {
                        path.push(par);
                        cur = parent.1.unwrap();
                    } else {
                        break;
                    }
                }
                path.reverse();
                return Some(path)
            }

            for relation in &current_occurrence.relations {
                if !visited.iter().map(|node| node.0).any(|occ_id| occ_id == relation.other_occurrence) {
                    queue.push_back(relation.other_occurrence);
                    visited.push((relation.other_occurrence, Some(current_occurrence_id)));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{dbobjects::{reference::TableReference, schema::relationgraph::{self, graph::RelationGraph, table_occurrence::TableOccurrence}}, emulator3::Emulator};


    #[test]
    fn build_path() {

        let mut env = Emulator::new();
        let db = env.load_file("test_data/cad_files/multi_file_solution/quotes.cad");
        let node1 = db.file.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 1).unwrap();
        let node2 = db.file.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 4).unwrap();
        let graph = &db.file.schema.relation_graph;
        let path = graph.get_path(node1.id, node2.id);
        assert_eq!(path, Some(vec![1, 5, 4]));
        let node1 = db.file.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 1).unwrap();
        let node2 = db.file.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 2).unwrap();
        let path = graph.get_path(node1.id, node2.id);
        assert_eq!(path, Some(vec![1, 2]));
        let node1 = db.file.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 5).unwrap();
        let node2 = db.file.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 3).unwrap();
        let path = graph.get_path(node1.id, node2.id);
        assert_eq!(path, None);
    }
}
