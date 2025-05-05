
use crate::token::SourceLoc;

pub struct KeyValueBlock {
    entries: Vec<KeyValueEntry>
}

impl KeyValueBlock {
    pub fn new() -> Self {
        Self {
            entries: vec![]
        }
    }

    pub fn add(&mut self, entry: KeyValueEntry) {
        self.entries.push(entry);
    }
}

pub enum CadmusObject {
    Table { id: u32, name: String },
    Field { id: u32, name: String },
    TableOccurrence { id: u32, name: String },
    ValueList { id: u32, name: String },
    Relation { id: u32 },
}

pub enum Key {
    TopLevelObject(CadmusObject),
    Name(String),
}

pub struct KeyValueEntry {
    key: Key,
    value: BlockValue,
    location: SourceLoc,
}

impl KeyValueEntry {
    pub fn new(key: Key, location: SourceLoc, value: BlockValue) -> Self {
            Self {
                key,
                location,
                value,
            }
        }
    }

pub enum BlockValue {
    Literal(String),
    Expression(String),
    Block(KeyValueBlock),
    Empty,
}

#[cfg(test)]
mod tests {
    use crate::token::SourceLoc;
    use super::{KeyValueBlock, KeyValueEntry, Key, CadmusObject, BlockValue};

    #[test]
    fn blocks() {
        let _ = KeyValueBlock {
            entries: vec![
                KeyValueEntry {
                    key: Key::TopLevelObject(CadmusObject::Table { id: 1, name: String::from("Quotes") }),
                    location: SourceLoc::new(1, 1),
                    value: BlockValue::Block(KeyValueBlock { entries: vec![

                    ] })
                }
            ]
        };
    }
}
