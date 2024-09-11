use std::cmp::Ordering;

#[derive(PartialEq, Eq, Ord, Debug)]
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

impl PartialOrd for HBAMPath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        for (a, b) in self.components
            .iter()
                .zip(&other.components)
                .map(|p| (p.0.parse::<usize>().unwrap_or(0), p.1.parse::<usize>().unwrap_or(0))) {
                    println!("a: {:?}, b: {:?}", a, b);
                    if a > b {
                        println!("{:?} > {:?}", a, b);
                        return Some(Ordering::Greater);
                    } else if b > a {
                        println!("{:?} > {:?}", b, a);
                        return Some(Ordering::Less);
                    }
        }
        if self.components.len() > other.components.len() {
            return Some(Ordering::Greater);
        } else if self.components.len() < other.components.len() {
            return Some(Ordering::Less);
        }
        return None
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
        assert!(HBAMPath::new(vec!["3", "17"]) > HBAMPath::new(vec!["3"]));
        assert!(HBAMPath::new(vec!["3", "17"]).partial_cmp(&HBAMPath::new(vec!["3"])) == Some(Ordering::Greater));
        assert!(HBAMPath::new(vec!["3", "17"]).partial_cmp(&HBAMPath::new(vec!["17"])) == Some(Ordering::Less));
    }
}
