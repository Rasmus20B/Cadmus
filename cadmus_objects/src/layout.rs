
use super::{file::File, reference::TableOccurrenceReference};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Layout {
    pub id: u32,
    pub name: String,
    pub occurrence: TableOccurrenceReference,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LayoutAttribute {
}

impl Layout {
    pub fn to_cad(&self, file: &File) -> String {
        format!("layout %{} {} : {} = {{}}",
            self.id,
            self.name,
            file.schema.relation_graph.nodes.iter()
            .inspect(|oc| println!("{}::{} == {}::{}?", self.occurrence.table_occurrence_id, self.name, oc.id, oc.name))
            .find(|oc| oc.id == self.occurrence.table_occurrence_id)
            .unwrap().name)
    }
}
