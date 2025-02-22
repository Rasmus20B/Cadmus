
use std::collections::BTreeMap;

use super::database::Database;

use super::window::Window;

pub struct WindowMgr {
    pub windows: BTreeMap<u32, Window>,
}

impl WindowMgr {
    pub fn new() -> Self {
        Self {
            windows: BTreeMap::new(),
        }
    }

    pub fn add_window(&mut self, database: &Database) -> u32 {
        let layout_id = database.file.layouts.iter().inspect(|l| println!("Layout: {}::{}::{:?}", l.id, l.name, l.occurrence)).min_by(|a, b| a.id.cmp(&b.id)).unwrap().id;
        println!("Chosen: {}", layout_id);
        let window_id = self.windows.len() as u32;
        self.windows.insert(window_id,
            Window::new()
                .id(window_id)
                .name(database.file.name.clone())
                .database(database.file.name.clone())
                .layout_id(layout_id)
            );
        window_id
    }
}
