
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

        for externs in db.file.data_sources.clone() {
            self.database_mgr.load_file(&externs.paths[0]);
        }
        self.database_mgr.databases.get(path).unwrap()
    }

    pub fn run_test_with_file(&mut self, test_name: &str, path: &str) -> Result<(), EmulatorErr>{
        self.state.active_database = path.to_string();
        let file = &self.load_file(path).file.tests;
        let test = match file
            .iter()
            .find(|search| &search.name == test_name) {
                Some(inner) => inner.clone(),
                None => return Err(EmulatorErr::UnknownTest(test_name.to_string()))
        };
        println!("Running test: {} on file: {}", test.name, path);
        //for instr in &test.instructions {
        //    println!("{:?}", instr);
        //}
        for (name, _) in &self.database_mgr.databases {
            println!("{}", name);
        }
        let status = self.script_mgr.run_script(test.clone(), ManagerRefs { window_mgr: &mut self.window_mgr, database_mgr: &mut self.database_mgr },  &mut self.state);
        Ok(())
    }
}

impl Host for Emulator {
    fn open_file(&mut self) { todo!() }
    fn run_test(&mut self, test_name: &str) { todo!() }
    fn run_test_on_file(&mut self, filename: &str, test_name: &str) { 
        self.run_test_with_file(test_name, filename);
    }
    fn step(&mut self) { todo!() }
}




