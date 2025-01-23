
use super::{window::Window, database::Database};

pub struct WindowMgr {
    current_idx: usize,
    windows: Vec<Window>,
}

impl WindowMgr {
    pub fn new() -> Self {
        Self {
            current_idx: 0,
            windows: vec![],
        }
    }

    pub fn select_window_by_name(&mut self, name: &str) {
        self.current_idx = self.windows
            .iter()
            .position(|w| w.name == name)
            .unwrap_or(self.current_idx)
    }

    pub fn current_window(&self) -> &Window {
        &self.windows[self.current_idx]
    }

    pub fn add_window(&mut self, file_: String, name: String, db: &Database) {
        self.windows.push(Window::new(self.windows.len() as u32, file_, name, db))
    }
}
