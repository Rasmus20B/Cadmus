use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Ord, Debug, Clone)]
pub struct HBAMPath {
    pub components: Vec<String>,
}

impl HBAMPath {
    pub fn new<S: Into<String>>(components: Vec<S>) -> Self {
        Self {
            components: components.into_iter().map(|s| s.into()).collect()
        }
    }
}

impl From<&[&str]> for HBAMPath {
    fn from(components: &[&str]) -> Self {
        Self {
            components: components.iter().map(|s| s.to_string()).collect()
        }
    }
}

impl From<String> for HBAMPath {
    fn from(components: String) -> Self {
        let components_ = components
            .split(',')
            .map(|s| s.trim().parse::<usize>().expect("Unable to parse directory component.").to_string())
            .collect::<Vec<String>>();

        HBAMPath::new(components_)
    }
}

impl PartialOrd for HBAMPath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.components.is_empty() && other.components.is_empty() { return Some(Ordering::Equal) }
        if self.components.is_empty() && !other.components.is_empty() { return Some(Ordering::Less) }
        if !self.components.is_empty() && other.components.is_empty() { return Some(Ordering::Greater) }

        let path1 = self.components.iter()
            .map(|component| component.parse::<usize>().expect("Unable to parse path."))
            .collect::<Vec<usize>>();

        let path2 = other.components.iter()
            .map(|component| component.parse::<usize>().expect("Unable to parse path."))
            .collect::<Vec<usize>>();

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

    use crate::hbam::path::HBAMPath;

    #[test]
    fn path_cmp() {
        assert!(HBAMPath::new(vec!["3", "17"]) > HBAMPath::new(vec!["2", "45"]));
        assert!(HBAMPath::new(vec!["1", "17", "19"]) < HBAMPath::new(vec!["2", "45"]));
        assert!(HBAMPath::new(vec!["3", "17"]) != HBAMPath::new(vec!["3"]));
        assert!(HBAMPath::new(vec!["3", "17"]).partial_cmp(&HBAMPath::new(vec!["3"])) == Some(Ordering::Equal));
        assert!(HBAMPath::new(vec!["3", "17"]).partial_cmp(&HBAMPath::new(vec!["17"])) == Some(Ordering::Less));
    }
}
