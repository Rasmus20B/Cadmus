use crate::dbobjects::reference::TableReference;

use super::relation::Relation;

pub struct TableOccurrence {
    name: String,
    base: TableReference,
}
