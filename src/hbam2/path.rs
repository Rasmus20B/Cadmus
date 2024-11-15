use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct HBAMPath {
    pub components: Vec<Vec<u8>>,
}

impl HBAMPath {
    pub fn new(components_: Vec<&[u8]>) -> Self {
        Self {
            components: components_.iter().map(|c| c.to_vec()).collect()
        }
    }

    pub fn contains(&self, contained: &HBAMPath) -> bool {
        if self.components.len() > contained.components.len() { return false }
        for (idx, _) in self.components.iter().enumerate() {
            if self.components[idx] != contained.components[idx] {
                return false
            }
        }
        true
    }
}

impl PartialOrd for HBAMPath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.components.is_empty() && other.components.is_empty() { return Some(Ordering::Equal) }
        if self.components.is_empty() && !other.components.is_empty() { return Some(Ordering::Less) }
        if !self.components.is_empty() && other.components.is_empty() { return Some(Ordering::Greater) }

        let path1 = &self.components;

        let path2 = &other.components;

        for i in 0..path1.len() {
            if i >= path2.len() { return Some(Ordering::Equal) }
            if path1[i] > path2[i] { return Some(Ordering::Greater) }
            else if path1[i] < path2[i] { return Some(Ordering::Less) }
        }
        Some(Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use crate::hbam2::path::HBAMPath;

    #[test]
    fn path_cmp() {
        assert!(HBAMPath::new(vec![&[3], &[17]]) > HBAMPath::new(vec![&[2], &[45]]));
        assert!(HBAMPath::new(vec![&[1], &[17], &[19]]) < HBAMPath::new(vec![&[2], &[45]]));
        assert!(HBAMPath::new(vec![&[3], &[17]]) != HBAMPath::new(vec![&[3]]));
        assert!(HBAMPath::new(vec![&[3], &[17]]).partial_cmp(&HBAMPath::new(vec![&[3]])) == Some(Ordering::Equal));
        assert!(HBAMPath::new(vec![&[3], &[17]]).partial_cmp(&HBAMPath::new(vec![&[17]])) == Some(Ordering::Less));
        assert!(HBAMPath::new(vec![]).partial_cmp(&HBAMPath::new(vec![&[3], &[17]])) == Some(Ordering::Less));
    }
}
