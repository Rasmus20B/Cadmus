
use std::rc::Rc;
use std::collections::HashMap;
use super::{database::Database, record::Record, layout::Layout, find::FoundSet};


pub enum WindowStyle {
    Document,
    FloatingDocument,
    Dialog,
    Card,
}

pub struct Window {
    pub id: u32,
    pub name: String,
    pub file: String,
    pub layout: Rc<Layout>,
    pub style: WindowStyle,
    pub found_sets: HashMap<usize, FoundSet>,
}

impl Window {

    pub fn new(id_: u32, file_: String, name_: String, db: &Database) -> Self {
        let layout = db.layouts.iter().min_by_key(|a| a.id).unwrap();
        let occurrence = db.table_occurrences.iter().find(|o| o.id == layout.table_occurrence_id).unwrap();
        let found_set = occurrence.get_records();
        let mut found_sets_ = HashMap::new();
        found_sets_.insert(occurrence.id, found_set);
        Self {
            id: id_,
            file: file_,
            name: name_,
            layout: Rc::clone(layout),
            style: WindowStyle::Document,
            found_sets: HashMap::new(),
        }
    }

    pub fn get_table_occurrence_id(&self) -> usize {
        self.layout.clone().table_occurrence_id as usize
    }

    pub fn get_current_found_set(&self) -> Option<&FoundSet> {
        self.found_sets.get(&self.get_table_occurrence_id())
    }
    pub fn get_current_record_ref(&self) -> Option<&Record> {
        self.get_current_found_set()?
            .current_record()
    }
}
