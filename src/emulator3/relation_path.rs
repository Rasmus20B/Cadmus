use std::collections::{HashMap, VecDeque};

use crate::dbobjects::schema::relationgraph::{graph::RelationGraph, table_occurrence::TableOccurrence};


fn get_path(node1: u32, node2: u32, graph: &RelationGraph) -> Vec<u32> {
    let mut queue = VecDeque::new();
    let mut visited = vec![];
    let start = graph.nodes.iter()
        .find(|node| node.id == node1)
        .unwrap();

    let mut path = vec![];

    visited.push((node1, None));
    queue.push_back(node1);

    while !queue.is_empty() {
        let current_occurrence_id = queue.pop_front().unwrap();
        let current_occurrence = graph.nodes.iter()
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
            return path
        }

        for relation in &current_occurrence.relations {
            if !visited.iter().map(|node| node.0).any(|occ_id| occ_id == relation.other_occurrence) {
                queue.push_back(relation.other_occurrence);
                visited.push((relation.other_occurrence, Some(current_occurrence_id)));
            }
        }
    }
    path
}

#[cfg(test)]
mod tests {
    use crate::{dbobjects::{reference::TableReference, schema::relationgraph::{self, graph::RelationGraph, table_occurrence::TableOccurrence}}, emulator3::Emulator};

    use super::get_path;

    #[test]
    fn build_path() {

        let mut env = Emulator::new();
        env.load_file("test_data/cad_files/multi_file_solution/quotes.cad");
        let db = env.database_mgr.databases.get("test_data/cad_files/multi_file_solution/quotes.cad").unwrap();
        let node1 = db.file.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 1).unwrap();
        let node2 = db.file.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 4).unwrap();
        let path = get_path(node1.id, node2.id, &db.file.schema.relation_graph);
        assert_eq!(path, vec![1, 5, 4]);
        let node1 = db.file.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 1).unwrap();
        let node2 = db.file.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 2).unwrap();
        let path = get_path(node1.id, node2.id, &db.file.schema.relation_graph);
        assert_eq!(path, vec![1, 2]);
    }
}
