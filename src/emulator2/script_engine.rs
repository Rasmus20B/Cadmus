
use super::{
    database::Database,
    database_mgr::DatabaseMgr,
    window_mgr::WindowMgr
};

use crate::schema::Script;
use crate::fm_script_engine::fm_script_engine_instructions::{ScriptStep, Instruction};
use std::rc::Rc;

pub enum StepResult {
    EndOfExecution
}

pub struct ScriptEngine {
    instr_ptrs: Vec<(u32, u32)>,
    variables: Vec<Vec<String>>,
    scripts: Vec<Rc<Script>>,
}

impl ScriptEngine {
    pub fn step(&mut self, dbs: &mut DatabaseMgr, window_mgr: &mut WindowMgr) -> Result<(), StepResult>  {
        // Set field test

        let step = self.next_instruction();
        if step.is_none() { return Err(StepResult::EndOfExecution) }
        let step = step.unwrap();

        let cur_window = window_mgr.current_window();
        match step.opcode {
            Instruction::SetField => {
                let cur_record = match cur_window.get_current_record_ref() {
                    Some(inner) => inner,
                    None => {
                        eprintln!("no records found.");
                        return Ok(());
                    }
                };

                /* Resolve regardless of which database, table, etc we are looking at. */
                // Looking at the table occurrence name, then look up the data source from that
                // (That means we store data source (maybe ID) and table in occurrence.
                // We then resolve occurrence through relationship graph
                // we then get a ref to the the first record in that occurrence. 
                // Then we call set like: vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
                cur_record.set_field(&step.switches[0], &step.switches[1]);
            },
            Instruction::SelectWindow => {
                window_mgr.select_window_by_name(&step.switches[0]);
            }
            _ => {
                eprintln!("Unknown Opcode.");
            }
        }
        return Ok(())
    }

    fn cur_script(&self) -> Option<Rc<Script>> {
        self.scripts.last().cloned()
    }

    fn next_instruction(&self) -> Option<ScriptStep> {
        let ip = match self.instr_ptrs.last() {
            Some(inner) => inner,
            None => return None
        };

        let script = match self.scripts.get(ip.0 as usize) {
            Some(inner) => inner,
            None => return None,
        };

        script.instructions.get(ip.1 as usize).cloned()
        
    }
}
