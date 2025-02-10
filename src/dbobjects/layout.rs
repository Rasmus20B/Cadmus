
use super::reference::TableOccurrenceReference;

#[derive(Debug, PartialEq, Eq)]
pub struct Layout {
    pub id: u32,
    pub name: String,
    pub occurrence: TableOccurrenceReference,
}
