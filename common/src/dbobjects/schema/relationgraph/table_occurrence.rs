use crate::dbobjects::reference::TableReference;

use super::relation::Relation;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TableOccurrence {
    pub id: u32,
    pub name: String,
    pub base: TableReference,
    pub relations: Vec<Relation>
}
