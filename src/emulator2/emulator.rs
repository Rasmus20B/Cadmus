
use super::{
    database::Database,
    database_mgr::DatabaseMgr,
    script_engine::ScriptEngine, 
    table::Table,
    window_mgr::WindowMgr,
};

use crate::schema::Schema;

pub struct Emulator {
   database_mgr: DatabaseMgr,
   script_engine: ScriptEngine,
   window_mgr: WindowMgr,
}

impl Emulator {
    fn start(&mut self, schema: &Schema) {
    }

    fn step(&mut self) {
        loop {
            let res = self.script_engine.step(&mut self.database_mgr, &mut self.window_mgr);
            if res.is_err() { break }
        }
    }
}
