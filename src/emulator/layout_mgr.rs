use std::collections::HashMap;

pub struct LayoutMgr {
    layout_tableoccurrence_lookup: HashMap::<String, usize>,
}

impl LayoutMgr {
    pub fn new() -> Self {
        Self {
            layout_tableoccurrence_lookup: HashMap::new(),
        }
    }

    pub fn add_mapping(&mut self, layout_name: String, table_occurrence: usize) {
        self.layout_tableoccurrence_lookup.insert(layout_name, table_occurrence);
    }

    pub fn lookup(&self, layout_name: String) -> Option<usize> {
        self.layout_tableoccurrence_lookup.get(&layout_name).copied()
    }
}
