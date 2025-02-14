
mod database;
mod database_mgr;
mod record_store;
mod script_mgr;
mod window_mgr;

use super::emulator3::database_mgr::DatabaseMgr;
use super::emulator3::script_mgr::ScriptMgr;
use super::emulator3::window_mgr::WindowMgr;

pub struct Emulator {
    database_mgr: DatabaseMgr,
    script_mgr: ScriptMgr,
    window_mgr: WindowMgr,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            database_mgr: DatabaseMgr::new(),
            script_mgr: ScriptMgr::new(),
            window_mgr: WindowMgr::new(),
        }
    }

    pub fn start_from_file(&mut self, path: String) {
        self.database_mgr.load_file(path);
    }
}
