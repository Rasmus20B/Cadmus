
mod context;
mod database;
mod database_mgr;
mod find;
mod record_store;
mod script_mgr;
mod window_mgr;
mod window;

use super::emulator3::database_mgr::DatabaseMgr;
use super::emulator3::database::Database;
use super::emulator3::script_mgr::ScriptMgr;
use super::emulator3::window_mgr::WindowMgr;

use std::path::{Path, PathBuf};

use crate::shell::Host;

struct ManagerRefs<'a> {
    pub database_mgr: &'a mut DatabaseMgr,
    pub window_mgr: &'a mut WindowMgr,
}

pub enum EmulatorErr {
    UnknownTest(String),
    UnknownFile(String),
}

#[derive(Debug, Clone)]
pub struct EmulatorState {
    pub active_database: String,
    pub active_window: u32,
}

impl EmulatorState {
    pub fn new() -> Self {
        Self {
            active_database: String::new(),
            active_window: 0,
        }
    }
}

pub struct Emulator {
    database_mgr: DatabaseMgr,
    script_mgr: ScriptMgr,
    window_mgr: WindowMgr,
    state: EmulatorState,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            database_mgr: DatabaseMgr::new(),
            script_mgr: ScriptMgr::new(),
            window_mgr: WindowMgr::new(),
            state: EmulatorState::new(),
        }
    }

    pub fn load_file(&mut self, path: &str) -> &Database {
        let db = self.database_mgr.load_file(path);
        self.state.active_window = self.window_mgr.add_window(db);
        let window = self.window_mgr.windows.get_mut(&self.state.active_window).unwrap();
        window.init_found_sets(db);
        // For now, we will assume all external files are needed as soon
        // as the specified file is opened.

        let working_dir_str = db.file.working_dir.clone();
        let working_dir = Path::new(&working_dir_str);
        for externs in db.file.data_sources.clone() {
            let ex = self.database_mgr.load_file(&(working_dir.to_str().unwrap().to_string() + "/" + &externs.paths[0]));
            self.window_mgr.add_window(ex);
        }
        self.database_mgr.databases.get(path).unwrap()
    }

    pub fn run_test_on_file(&mut self, test_name: &str, path: &str) -> Result<(), EmulatorErr>{
        self.state.active_database = path.to_string();
        let tests = &self.load_file(path).file.tests;
        let test = match tests
            .iter()
            .find(|search| &search.name == test_name) {
                Some(inner) => inner.clone(),
                None => return Err(EmulatorErr::UnknownTest(test_name.to_string()))
        };
        println!("Running test: {} on file: {}", test.name, path);
        let status = self.script_mgr.run_script(test.clone(), ManagerRefs { window_mgr: &mut self.window_mgr, database_mgr: &mut self.database_mgr },  &mut self.state);
        Ok(())
    }
}

impl Host for Emulator {
    fn open_file(&mut self) { todo!() }

    fn run_tests(&mut self, filename: &str) { 
        let tests = self.load_file(filename).file.tests.iter().map(|test| test.name.clone()).collect::<Vec<String>>();

        for test in tests {
            self.run_test_from_file(filename, &test);
        }
    }

    fn run_test_from_file(&mut self, filename: &str, test_name: &str) { 
        let _ = self.run_test_on_file(test_name, filename);
    }
    fn step(&mut self) { todo!() }
}




