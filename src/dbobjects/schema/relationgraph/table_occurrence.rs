use crate::dbobjects::reference::TableReference;

use super::relation::Relation;

pub struct TableOccurrence {
    pub id: u32,
    pub name: String,
    pub base: TableReference,
    pub relations: Vec<Relation>
}
