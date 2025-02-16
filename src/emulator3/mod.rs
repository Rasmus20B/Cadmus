
mod database;
mod database_mgr;
mod record_store;
mod script_mgr;
mod window_mgr;
mod window;

use super::emulator3::database_mgr::DatabaseMgr;
use super::emulator3::script_mgr::ScriptMgr;
use super::emulator3::window_mgr::WindowMgr;
use super::emulator3::window::Window;

use crate::dbobjects::scripting::script::Script;

pub struct Emulator<'a> {
    database_mgr: DatabaseMgr,
    script_mgr: ScriptMgr<'a>,
    window_mgr: WindowMgr,
}

impl<'a> Emulator<'a> {
    pub fn new() -> Self {
        Self {
            database_mgr: DatabaseMgr::new(),
            script_mgr: ScriptMgr::new(),
            window_mgr: WindowMgr::new(),
        }
    }

    pub fn load_file(&mut self, path: String) {
        let db = self.database_mgr.load_file(path);
        let window = self.window_mgr.add_window(db);
        // For now, we will assume all external files are needed as soon
        // as the specified file is opened.

        for externs in db.file.data_sources.clone() {
            self.database_mgr.load_file(externs.paths[0].clone());
        }
    }

    pub fn run_test_with_file(&mut self, test: &'a Script, path: String) {
        println!("Running test: {} on file: {}", test.name, path);
        let status = self.script_mgr.run_script(&test, &mut self.database_mgr, &mut self.window_mgr);
        println!("{:?}", status);
    }
}
