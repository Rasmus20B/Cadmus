
use std::collections::BTreeMap;

use super::database::Database;

use super::window::Window;

pub struct WindowMgr {
    windows: BTreeMap<u32, Window>,
    current: Option<u32>,
}

impl WindowMgr {
    pub fn new() -> Self {
        Self {
            windows: BTreeMap::new(),
            current: None,
        }
    }

    pub fn add_window(&mut self, database: &Database) {
        let layout_id = database.file.layouts.get(0).unwrap().id;
        let window_id = self.windows.len() as u32;
        self.windows.insert(window_id,
            Window::new()
                .id(window_id)
                .name(database.file.name.clone())
                .database(database.file.name.clone())
                .layout_id(layout_id)
            );
    }
}
