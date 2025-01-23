
use super::{
    database::Database,
    database_mgr::DatabaseMgr,
    window_mgr::WindowMgr,
};

use crate::schema::Script;
use crate::fm_script_engine::fm_script_engine_instructions::{ScriptStep, Instruction};
use std::rc::Rc;

pub enum StepResult {
    EndOfExecution
}

pub struct ScriptEngine {
    instr_ptrs: Vec<(Script, u32)>,
    variables: Vec<Vec<String>>,
}

impl  ScriptEngine {
    pub fn new() -> Self {
        Self {
            instr_ptrs: vec![],
            variables: vec![],
        }
    }

    pub fn perform_script(&mut self, script: &Script, db_mgr: &mut DatabaseMgr, win_mgr: &mut WindowMgr) {
        self.instr_ptrs.push((script.clone(), 0));
        loop {
            let res = self.step(db_mgr, win_mgr);
            if res.is_err() { break }
        }
    }

    pub fn step(&mut self, dbs: &mut DatabaseMgr, window_mgr: &mut WindowMgr) -> Result<(), StepResult>  {
        let step = self.next_instruction();
        println!("STEP: {:?}", step);
        if step.is_none() { return Err(StepResult::EndOfExecution) }
        let step = step.unwrap();

        let cur_window = window_mgr.current_window();
        match step.opcode {
            Instruction::NewRecordRequest => {
                let table = dbs.databases.get_mut(&cur_window.file).unwrap().table_occurrences[cur_window.layout.table_occurrence_id].table.id;
                dbs.add_record(cur_window.file.clone(), table);
            }
            Instruction::GoToLayout => {
            }
            Instruction::SetField => {
                let cur_record = match cur_window.get_current_record_ref() {
                    Some(inner) => inner,
                    None => {
                        self.instr_ptrs.last_mut().unwrap().1 += 1;
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
            Instruction::PerformScript => {
                let db = dbs.databases.get(&window_mgr.current_window().file).unwrap();
                let n = step.switches[0].len();
                let script_name = &step.switches[0][1..n-1];
                let script = db.scripts.iter().find(|s| s.name == script_name).unwrap();

                self.instr_ptrs.push((script.clone(), 1));
                return Ok(())
            }
            Instruction::SelectWindow => {
                window_mgr.select_window_by_name(&step.switches[0]);
            }
            _ => {
                eprintln!("Unknown Opcode.");
            }
        }

        self.instr_ptrs.last_mut().unwrap().1 += 1;
        return Ok(())
    }

    fn cur_script(&self) -> Option<&Script> {
        match self.instr_ptrs.last() {
            Some((a, _)) => Some(a),
            None => None,
        }
    }

    fn next_instruction(&self) -> Option<ScriptStep> {
        let ip = match self.instr_ptrs.last() {
            Some(inner) => inner,
            None => return None
        };

        let script = match self.instr_ptrs.last() {
            Some(inner) => inner.0.clone(),
            None => return None,
        };

        script.instructions.get(ip.1 as usize).cloned()
        
    }
}
