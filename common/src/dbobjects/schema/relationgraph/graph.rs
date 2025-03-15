
use std::collections::{HashMap, HashSet};
use crate::dbobjects::file::File;

use super::{table_occurrence::TableOccurrence, relation::Relation};
use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq, Clone)]
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

    pub fn to_cad(&self, file: &File, externs: &HashMap<usize, File>) -> String {
        let mut buffer = String::new();
        let mut relation_buffer = String::new();
        let mut relation_set = HashSet::new();
        for node in &self.nodes {
            if node.base.data_source == 0 {
                println!("TO: {}", node.name);
                let table = file.schema.tables.iter().find(|table| table.id == node.base.table_id).unwrap();
                buffer.push_str(&format!("table_occurrence %{} {} : {}\n", node.id, node.name, table.name));
            } else {
                let ds = externs.get(&(node.base.data_source as usize)).unwrap();
                let ds_name = file.data_sources.iter().find(|ds| ds.id == node.base.data_source).unwrap().name.clone();
                let table = ds.schema.tables.iter().find(|table| table.id == node.base.table_id).unwrap();
                buffer.push_str(&format!("table_occurrence %{} {} : {}::{}\n", node.id, node.name, ds_name, table.name));
            }

            for relation in &node.relations {
                if relation_set.contains(&relation.id) { continue }
                relation_set.insert(relation.id);

                let other_occurrence = file.schema.relation_graph.nodes.iter().find(|node| node.id == relation.other_occurrence).unwrap();
                let other_occurrence_table = if other_occurrence.base.data_source == 0 {
                    file.schema.tables.iter().find(|table| table.id == other_occurrence.base.table_id).unwrap()
                } else {
                    let ds = file.data_sources.iter()
                        .find(|ds| ds.id == other_occurrence.base.data_source)
                        .unwrap();
                    externs.get(&(ds.id as usize)).unwrap().schema.tables.iter().find(|table| table.id == other_occurrence.base.table_id).unwrap()
                };

                let this_occurrence_table = if node.base.data_source == 0 {
                    file.schema.tables.iter().find(|table| table.id == node.base.table_id).unwrap()
                } else {
                    let ds = file.data_sources.iter()
                        .find(|ds| ds.id == node.base.data_source)
                        .unwrap();
                    externs.get(&(ds.id as usize)).unwrap().schema.tables.iter().find(|table| table.id == node.base.table_id).unwrap()
                };

                if relation.criteria.len() == 1 {
                    relation_buffer.push_str(&format!("relation %{} = {}::{} {} {}::{}\n",
                            relation.id,
                            node.name,
                            this_occurrence_table.fields.get(&relation.criteria[0].field_self).unwrap().name,
                            relation.criteria[0].comparison,
                            other_occurrence_table.fields.get(&relation.criteria[0].field_other).unwrap().name,
                            other_occurrence.name,
                            ));
                } else {
                    relation_buffer.push_str(&format!("relation %{} = {{\n", relation.id));
                    for crit in &relation.criteria {
                        buffer.push_str(&format!("    {}::{} {} {}::{},\n", 
                                node.name,
                                this_occurrence_table.fields.get(&crit.field_self).unwrap().name,
                                crit.comparison,
                                other_occurrence.name,
                                other_occurrence_table.fields.get(&crit.field_other).unwrap().name
                        )
                    );
                    }
                    relation_buffer.push_str("}\n");
                }
            }
        }

        buffer.push_str("\n\n");
        buffer.push_str(&mut relation_buffer);

        buffer
    }
}

#[cfg(test)]
mod tests {
    use crate::dbobjects::{reference::TableReference, schema::relationgraph::{self, graph::RelationGraph, table_occurrence::TableOccurrence}};
    use crate::cadlang::compiler::compile_to_file;
    use std::path::Path;

    #[test]
    fn build_path() {
        let db = compile_to_file(Path::new("test_data/cad_files/multi_file_solution/quotes.cad")).unwrap();
        let node1 = db.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 1).unwrap();
        let node2 = db.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 4).unwrap();
        let graph = &db.schema.relation_graph;
        let path = graph.get_path(node1.id, node2.id);
        assert_eq!(path, Some(vec![1, 5, 4]));
        let node1 = db.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 1).unwrap();
        let node2 = db.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 2).unwrap();
        let path = graph.get_path(node1.id, node2.id);
        assert_eq!(path, Some(vec![1, 2]));
        let node1 = db.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 5).unwrap();
        let node2 = db.schema.relation_graph.nodes.iter()
            .find(|occurrence| occurrence.id == 3).unwrap();
        let path = graph.get_path(node1.id, node2.id);
        assert_eq!(path, None);
    }
}








