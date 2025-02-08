
use super::{
    database::Database,
    database_mgr::DatabaseMgr,
    window_mgr::WindowMgr,
};

use crate::dbobjects::{reference::*, calculation::Calculation, scripting::{instructions::*, arguments::*, script::*}};

pub enum StepResult {
    EndOfExecution
}

pub struct ScriptEngine {
    instr_ptrs: Vec<(Script, u32)>,
    variables: Vec<Vec<String>>,
}

impl ScriptEngine {
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
        match step {
            Instruction::NewRecordRequest => {
                let table = dbs.databases.get_mut(&cur_window.file).unwrap().table_occurrences[cur_window.layout.table_occurrence_id].table.id;
                dbs.add_record(cur_window.file.clone(), table);
            }
            Instruction::GoToLayout => {
            }
            Instruction::SetField { field, value } => {
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
                //cur_record.set_field(field, &step.switches[1]);
            },
            Instruction::PerformScript { script, args } => {
                let db = dbs.databases.get(&window_mgr.current_window().file).unwrap();
                let script_ref = match script {
                    ScriptSelection::FromList(name) => name,
                    ScriptSelection::ByCalculation(calc) => {
                        let name = calc.eval();
                        ScriptReference { 
                            data_source: 0,
                            script_id: db.scripts.iter().find(|script| script.name == name).unwrap().id as u32
                        }
                    }
                };
                let script = db.scripts.iter().find(|s| s.id == script_ref.script_id).unwrap();

                //self.instr_ptrs.push((script.clone(), 1));
                return Ok(())
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

    fn next_instruction(&self) -> Option<Instruction> {
        todo!()
        //let ip = match self.instr_ptrs.last() {
        //    Some(inner) => inner,
        //    None => return None
        //};
        //
        //let script = match self.instr_ptrs.last() {
        //    Some(inner) => inner.0.clone(),
        //    None => return None,
        //};
        //
        //script.instructions.get(ip.1 as usize).cloned()
        //
    }
}
