use core::fmt;
use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
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
                println!("{} != {}", self, contained);
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

        if path1.len() > path2.len() {
            for i in 0..path1.len() {
                if i >= path2.len() { return Some(Ordering::Equal) }
                if path1[i] > path2[i] { return Some(Ordering::Greater) }
                else if path1[i] < path2[i] { return Some(Ordering::Less) }
            }
        } else {
            for i in 0..path2.len() {
                if i >= path1.len() { return Some(Ordering::Equal) }
                if path2[i] > path1[i] { return Some(Ordering::Less) }
                else if path2[i] < path1[i] { return Some(Ordering::Greater) }
            }
        }
        Some(Ordering::Equal)
    }
}

impl PartialEq for HBAMPath {
   fn eq(&self, other: &Self) -> bool {
        let path1 = &self.components;
        let path2 = &other.components;

        if path1.len() > path2.len() {
            for i in 0..path1.len() {
                if i >= path2.len() { return true }
                if path1[i] > path2[i] { return false }
                else if path1[i] < path2[i] { return false }
            }
        } else {
            for i in 0..path2.len() {
                if i >= path1.len() { return true }
                if path2[i] > path1[i] { return false }
                else if path2[i] < path1[i] { return false }
            }
        }
        true
   }
}

impl Eq for HBAMPath {}

impl Ord for HBAMPath {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            Some(inner) => inner,
            None => Ordering::Greater
        }
    }
}

impl fmt::Display for HBAMPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for c in &self.components {
            result.push_str(format!("[{:?}].", c).as_str());
        }
        result.pop();
        write!(f, "{}", result)
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
        assert!(HBAMPath::new(vec![&[3], &[17]]) == HBAMPath::new(vec![&[3]]));
        assert!(HBAMPath::new(vec![&[3], &[17]]).partial_cmp(&HBAMPath::new(vec![&[3]])) == Some(Ordering::Equal));
        assert!(HBAMPath::new(vec![&[3], &[17]]).partial_cmp(&HBAMPath::new(vec![&[17]])) == Some(Ordering::Less));
        assert!(HBAMPath::new(vec![&[3], &[17], &[5]]).partial_cmp(&HBAMPath::new(vec![&[3], &[17]])) == Some(Ordering::Equal));
        assert!(HBAMPath::new(vec![&[128, 1], &[3], &[5]]).partial_cmp(&HBAMPath::new(vec![&[128, 1]])) == Some(Ordering::Equal));
        assert!(HBAMPath::new(vec![&[128, 1], &[3], &[5]]) == HBAMPath::new(vec![&[128, 1]]));
        assert!(HBAMPath::new(vec![&[128, 1]]) == HBAMPath::new(vec![&[128, 1], &[3], &[5]]));
        assert!(HBAMPath::new(vec![&[128, 2]]) >= HBAMPath::new(vec![&[128, 1]]));
        assert!(HBAMPath::new(vec![&[128, 1]]) <= HBAMPath::new(vec![&[128, 2]]));
        assert!(HBAMPath::new(vec![&[128, 1]]) < HBAMPath::new(vec![&[128, 2]]));
        assert!(HBAMPath::new(vec![&[128, 1], &[3]]) < HBAMPath::new(vec![&[128, 2]]));
        assert!(HBAMPath::new(vec![&[4], &[5], &[1], &[14]]) > HBAMPath::new(vec![&[4], &[5], &[1], &[13]]));
        assert!(HBAMPath::new(vec![&[4], &[5], &[1], &[14]]) >= HBAMPath::new(vec![&[4], &[5], &[1], &[13]]));
        assert!(HBAMPath::new(vec![]).partial_cmp(&HBAMPath::new(vec![&[3], &[17]])) == Some(Ordering::Less));
    }
}
