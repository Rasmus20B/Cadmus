
use std::rc::Rc;
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
    pub layout: Rc<Layout>,
    pub style: WindowStyle,
    pub found_set: FoundSet,
}

impl Window {
    pub fn get_current_record_ref(&self) -> Option<&Record> {
        self.found_set
            .current_record()
    }
}
