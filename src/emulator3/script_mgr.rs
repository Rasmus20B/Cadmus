
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

    pub loop_stack: Vec<u32>,
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
            loop_stack: vec![],
        }
    }

    pub fn get_var(&self, name: &str) -> Option<String> {
        self.variables.last()
            .unwrap()
            .iter()
            .find(|var| var.0.as_str() == name)
            .map(|var| var.1.clone())
            .or_else(|| None)
    }

    pub fn set_var(&mut self, name: &str, value: String) {
        let handle = self.variables.last_mut()
            .unwrap()
            .iter_mut()
            .find(|var| var.0.as_str() == name);

        match handle {
            Some(inner) => { inner.1 = value },
            None => { 
                self.variables
                    .last_mut()
                    .unwrap()
                    .push((name.to_string(), value));
            }
        }
    }

    pub fn run_script(&mut self, test: &'a Script, db_mgr: &mut DatabaseMgr, window_mgr: &mut WindowMgr, state: &mut EmulatorState) -> Result<(), ScriptErr> {
        self.program_counters.push((0, test));
        self.variables.push(vec![]);

        while let Some(mut ip) = self.program_counters.pop() {
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

                    let val = value.eval(&context).unwrap();
                    let rep = repetition.eval(&context).unwrap();


                    self.set_var(name, val);
                    ip.0 += 1;
                }
                Instruction::Loop => {
                    self.loop_stack.push(ip.0);
                    ip.0 += 1;
                }
                Instruction::EndLoop => {
                    ip.0 = self.loop_stack.pop().unwrap();
                }
                Instruction::ExitLoopIf { condition } => {
                    let context = EmulatorContext {
                        database_mgr: &*db_mgr,
                        variables: self.variables.last().unwrap(),
                        globals: &self.globals,
                        window_mgr: &*window_mgr,
                        state: &*state,
                    };
                    let exit = condition.eval(&context).unwrap();
                    if exit == "true" {
                        let mut depth = 1;
                        while ip.0 < ip.1.instructions.len() as u32 && depth != 0 {
                            ip.0 += 1;
                            match ip.1.instructions[ip.0 as usize].instruction {
                                Instruction::Loop => { depth += 1; },
                                Instruction::EndLoop => { depth -= 1; },
                                _ => {}
                            }
                        }
                        ip.0 += 1;
                    } else {
                        ip.0 += 1;
                    }
                },
                _ => {
                }
            }

            if ip.0 < ip.1.instructions.len() as u32 {
                self.program_counters.push(ip);
            }
        }


        Ok(())
    }

}
