
use super::database_mgr::DatabaseMgr;
use super::window_mgr::WindowMgr;
use super::EmulatorState;
use super::context::EmulatorContext;

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

    pub fn run_script(&mut self, test: &'a Script, db_mgr: &mut DatabaseMgr, window_mgr: &mut WindowMgr, state: &mut EmulatorState) -> Result<(), ScriptErr> {
        self.program_counters.push((1, test));

        while let Some(ip) = self.program_counters.last_mut() {
            let cur_instr = &ip.1.instructions[ip.0 as usize];

            match &cur_instr.instruction {
                Instruction::NewRecordRequest => {
                    ip.0 += 1;
                }
                Instruction::SetVariable { name, value, repetition } => {
                    let context = EmulatorContext {
                        database_mgr: &*db_mgr,
                        variables: self.variables.last().unwrap(),
                        globals: &self.globals,
                        window_mgr: &*window_mgr,
                        state: &*state,
                    };

                    value.eval(&context);
                    ip.0 += 1;
                }
                Instruction::Loop => {
                    ip.0 += 1;
                }
                Instruction::EndLoop => {
                    ip.0 += 1;
                }
                Instruction::ExitLoopIf { condition } => {
                    ip.0 += 1;
                },
                _ => {
                }
            }

            if ip.0 >= ip.1.instructions.len() as u32 {
                self.program_counters.pop();
            }
        }


        Ok(())
    }

}
