
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
        let layout_id = database.file.layouts
            .iter()
            .min_by(|a, b| a.id.cmp(&b.id))
            .unwrap().id;
        let window_id = self.windows.len() as u32;
        let mut tmp = Window::new()
                    .id(window_id)
                    .name(database.file.name.clone())
                    .database(database.file.name.clone())
                    .layout_id(layout_id);

        tmp.init_found_sets(database);
        self.windows.insert(window_id, tmp);
        window_id
    }
}
