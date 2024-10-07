use std::collections::{vec_deque, HashMap, VecDeque};

pub struct RelationMgr {
    graph: HashMap<usize, Vec<usize>>,
}

impl RelationMgr {
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, occ1: usize, occ2: usize) {
        let exists = self.graph.get(&occ1).unwrap_or(&vec![])
            .iter()
            .any(|x| *x == occ2);

        let graph_node = self.graph.get_mut(&occ1);
        if graph_node.is_none() {
            self.graph.insert(occ1, vec![occ2]);
        } else if !exists {
            graph_node.unwrap().push(occ2);
        }
    }
    pub fn add_relation(&mut self, occ1: usize, occ2: usize) {
        self.add_node(occ1, occ2);
        self.add_node(occ2, occ1);
    }

    pub fn get_path(&self, occ1: usize, occ2: usize) -> Option<Vec<usize>> {

        let mut visited = Vec::new();
        let mut queue = VecDeque::<(usize, Vec<usize>)>::new();
        queue.push_back((occ1, vec![occ1]));

        while !queue.is_empty() {
            let (cur, mut path) = queue.front().unwrap().clone();
            queue.pop_front();

            let list = self.graph.get(&cur);
            if list.is_none() {
                continue;
            }

            for relation in list.unwrap()
                .iter()
                .filter(|x| !visited.contains(x))
                .collect::<Vec<_>>() {
                if *relation == occ2 {
                    path.push(occ2);
                    return Some(path);
                } else {
                    let mut next_path = path.clone();
                    next_path.push(*relation);
                    queue.push_back((*relation, next_path));
                    visited.push(relation);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::RelationMgr;

    #[test]
    pub fn graph_test() {
        let mut mgr = RelationMgr::new();
        mgr.add_relation(1, 2);
        mgr.add_relation(2, 1);
        mgr.add_relation(2, 7);
        mgr.add_relation(1, 3);
        mgr.add_relation(8, 9);
        mgr.add_relation(5, 1);

        assert_eq!(mgr.get_path(1, 7), vec![1, 2, 7].into());
        assert_eq!(mgr.get_path(2, 1), vec![2, 1].into());
        assert_eq!(mgr.get_path(1, 9), None);
    }
}
