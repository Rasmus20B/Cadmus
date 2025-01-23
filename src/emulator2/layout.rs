
pub struct Layout {
    pub id: usize,
    pub name: String,
    pub table_occurrence_id: usize,
}

impl Layout {
    pub fn new(id_: usize, name_: String, table_occurrence_id_: usize) -> Self {
        Self {
            id: id_,
            name: name_,
            table_occurrence_id: table_occurrence_id_
        }
    }
}
