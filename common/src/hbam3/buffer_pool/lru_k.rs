use std::collections::HashMap;

use super::frame::FrameID;


#[derive(Default, Debug)]
pub struct Node {
    history: Vec<Option<usize>>,
    pub is_evictable: bool,
}

impl Node {
    pub fn new(k: usize) -> Self {
        Self {
            history: vec![None; k],
            is_evictable: false,
        }
    }

    pub fn add_history(&mut self, timestamp: usize) {
        if !self.history.contains(&None) {
            self.history.remove(0);
            self.history.push(Some(timestamp));
            return
        }

        if let Some(handle) = self.history.iter_mut()
            .find(|entry| entry.is_none()) {
            *handle = Some(timestamp);
            return;
        }

    }

    pub fn k_distance(&self, k: usize) -> usize {
        if self.history.contains(&None) { return usize::MAX }
        self.history.last().unwrap().unwrap() - self.history[0].unwrap()
    }
}

pub struct Replacer {
    node_store: HashMap<FrameID, Node>,
    current_timestamp: usize,
    current_size: usize,
    replacer_size: usize,
    k: usize,
}

impl Replacer {
    pub fn new(k: usize) -> Self {
        Self {
            node_store: HashMap::new(),
            current_timestamp: 0,
            current_size: 0,
            replacer_size: 0,
            k,
        }
    }
    pub fn evict(&mut self) -> Option<FrameID> {
        let Some(max) = self.node_store.iter()
            .filter(|n| n.1.is_evictable)
            .inspect(|node| println!("{:?}", node))
            .map(|node| if  node.1.history.contains(&None) { return usize::MAX } else { node.1.k_distance(self.k) })
            .max() else {
                return None
            };

        println!("max: {:?}", max);

        let frame_id = self.node_store.iter()
            .filter(|node| node.1.is_evictable && max == node.1.k_distance(self.k))
            .inspect(|n| println!("in max: {:?}", n))
            .map(|node| (node.0, node.1.history.iter().rev().find(|entry| entry.is_some()).unwrap()))
            .min_by(|a, b| a.1.cmp(b.1))
            .map(|n| n.0)
            .or(None).copied();

        self.remove(frame_id?);
        self.replacer_size -= 1;

        frame_id
    }

    pub fn record_access(&mut self, frame_id_: FrameID) {
        self.current_timestamp += 1;
        let handle = self.node_store.entry(frame_id_).or_insert(Node::new(self.k));
        handle.add_history(self.current_timestamp);
    }

    pub fn set_evictable(&mut self, frame_id_: FrameID, set_evictable: bool) {
        let Some(handle) = self.node_store.get_mut(&frame_id_) else {
            return;
        };
        let current = handle.is_evictable;
        if set_evictable == current { return }
        if set_evictable { self.replacer_size += 1 } else { self.replacer_size -= 1 };
        handle.is_evictable = set_evictable;
    }

    fn remove(&mut self, frame_id_: usize) {
        self.node_store.remove(&frame_id_);
    }

    pub fn len(&self) -> usize {
        self.replacer_size
    }
}

#[cfg(test)]
mod tests {
    use super::Replacer;

    #[test]
    fn test_basic_eviction() {
        let mut policy = Replacer::new(2);

        policy.record_access(1);
        policy.record_access(2);
        policy.record_access(3);
        policy.record_access(4);
        policy.record_access(5);
        policy.record_access(6);
        policy.set_evictable(1, true);
        policy.set_evictable(2, true);
        policy.set_evictable(3, true);
        policy.set_evictable(4, true);
        policy.set_evictable(5, true);
        policy.set_evictable(6, false);
        assert_eq!(policy.replacer_size, 5);

        policy.record_access(1);
        assert_eq!(policy.evict(), Some(2));
        assert_eq!(policy.evict(), Some(3));
        assert_eq!(policy.evict(), Some(4));
        assert_eq!(policy.len(), 2);

        policy.record_access(3);
        policy.record_access(4);
        policy.record_access(5);
        policy.record_access(4);
        policy.set_evictable(3, true);
        policy.set_evictable(4, true);

        assert_eq!(policy.len(), 4);
        assert_eq!(policy.evict(), Some(3));
        assert_eq!(policy.len(), 3);

        policy.set_evictable(6, true);
        assert_eq!(4, policy.len());
        assert_eq!(policy.evict(), Some(6));
        assert_eq!(3, policy.len());

        policy.set_evictable(1, false);

        assert_eq!(2, policy.len());
        assert_eq!(policy.evict(), Some(5));
        assert_eq!(1, policy.len());

        policy.record_access(1);
        policy.record_access(1);
        policy.set_evictable(1, true);
        assert_eq!(2, policy.len());

        assert_eq!(policy.evict(), Some(4));
        assert_eq!(1, policy.len());
        assert_eq!(policy.evict(), Some(1));
        assert_eq!(0, policy.len());

        policy.record_access(1);
        policy.set_evictable(1, false);
        assert_eq!(0, policy.len());

        assert_eq!(None, policy.evict());

        policy.set_evictable(1, true);
        assert_eq!(1, policy.len());
        assert_eq!(policy.evict(), Some(1));
        assert_eq!(0, policy.len());

        assert_eq!(policy.evict(), None);
        assert_eq!(0, policy.len());

        policy.set_evictable(6, false);
        policy.set_evictable(6, true);
        assert_eq!(0, policy.len());
    }
}
