
use super::reference::TableOccurrenceReference;

pub struct Layout {
    pub id: u32,
    pub name: String,
    pub occurrence: TableOccurrenceReference,
}
