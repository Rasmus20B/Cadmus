
use super::database_mgr::DatabaseMgr;
use super::window_mgr::WindowMgr;

use crate::dbobjects::scripting::{script::Script, instructions::Instruction};
use crate::dbobjects::calculation::Calculation;

pub struct ScriptMgr<'a> {
    pub program_counters: Vec<(u32, &'a Script)>,
    pub variables: Vec<Vec<(String, String)>>,
    pub globals: Vec<(String, String)>,

}

#[derive(Debug)]
pub enum ScriptErr {
}

impl<'a> ScriptMgr<'a> {
    pub fn new() -> Self {
        Self {
            program_counters: vec![],
            variables: vec![],
            globals: vec![],
        }
    }

    pub fn run_script(&mut self, test: &'a Script, db_mgr: &mut DatabaseMgr, window_mgr: &mut WindowMgr) -> Result<(), ScriptErr> {
        self.program_counters.push((1, test));

        while let Some(ip) = self.program_counters.last() {
            let cur_instr = &ip.1.instructions[ip.0 as usize];

            match &cur_instr.instruction {
                Instruction::SetVariable { name, value, repetition } => {
                }
                _ => {}
            }



            if ip.0 >= ip.1.instructions.len() as u32 {
                self.program_counters.pop();
            }
        }


        Ok(())
    }

}
