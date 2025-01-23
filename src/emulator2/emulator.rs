
use super::{
    database::Database,
    database_mgr::DatabaseMgr,
    data_source::*,
    window_mgr::WindowMgr,
    window::*,
    script_engine::ScriptEngine, 
    table::Table,
    layout::Layout,
    find::*,
};

use std::path::Path;
use std::rc::Rc;
use crate::schema::{Schema, Script};

pub struct Emulator {
   database_mgr: DatabaseMgr,
   script_engine: ScriptEngine,
   window_mgr: WindowMgr,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            database_mgr: DatabaseMgr::new(),
            script_engine: ScriptEngine::new(),
            window_mgr: WindowMgr::new(),
        }
    }

    pub fn open_file(&mut self, path: &Path) {
        let ftype = path.extension().expect("invalid filetype.");
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        match ftype.to_str() {
            Some("cad") => {
                let db = self.database_mgr.load_cadmus_file(path).unwrap();
                self.window_mgr.add_window(path.to_str().unwrap().to_string(), name.clone(), db);
                self.window_mgr.select_window_by_name(&name);
            },
            Some("fmp12") => {
            },
            _ => {}
        }
    }

    fn run_test(&mut self, test: &Script) {
        //self.script_engine.perform_script(&test, );
        let res = self.script_engine.perform_script(&test, &mut self.database_mgr, &mut self.window_mgr);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::cadlang;
    use std::fs::read_to_string;
    #[test]
    fn cad_startup_test() {
        let mut emulator = Emulator::new();
        emulator.open_file(Path::new("./test_data/cad_files/initial.cad"));
        assert_eq!(emulator.database_mgr.databases.len(), 1);
        assert_eq!(emulator.database_mgr.databases["./test_data/cad_files/initial.cad"].tables.len(), 3);

        let test_code = read_to_string(Path::new("./test_data/cad_files/initial_test.cad")).unwrap();
        let test = cadlang::compiler::compile_to_schema(test_code).unwrap().tests.get(&1).unwrap().script.clone();
        emulator.run_test(&test);
    }
}





