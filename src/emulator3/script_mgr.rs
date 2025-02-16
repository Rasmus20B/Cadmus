
use super::database_mgr::DatabaseMgr;
use super::window_mgr::WindowMgr;

use crate::dbobjects::scripting::script::Script;

pub struct ScriptMgr<'a> {
    pub program_counters: Vec<(u32, &'a Script)>,
}

#[derive(Debug)]
pub enum ScriptErr {
}

impl<'a> ScriptMgr<'a> {
    pub fn new() -> Self {
        Self {
            program_counters: vec![]
        }
    }

    pub fn run_script(&mut self, test: &'a Script, db_mgr: &mut DatabaseMgr, window_mgr: &mut WindowMgr) -> Result<(), ScriptErr> {
        self.program_counters.push((1, test));
        todo!()
    }

}
