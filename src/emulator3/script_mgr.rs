
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
                    let window = window_mgr.windows.get(&state.active_window).unwrap();

                    let occ = db_mgr.databases.get(&state.active_database)
                        .unwrap()
                        .file.layouts.iter()
                        .find(|search| search.id == window.layout_id).unwrap()
                        .occurrence.table_occurrence_id;

                    let table_id = db_mgr.databases.get(&state.active_database)
                        .unwrap().file.schema.relation_graph.nodes.iter()
                        .find(|search| search.id == occ)
                        .map(|found| found.base.table_id)
                        .unwrap();

                    let table = db_mgr.databases.get(&state.active_database)
                        .unwrap().file.schema.tables.iter()
                        .find(|search| search.id == table_id)
                        .unwrap().clone();

                    db_mgr.databases.get_mut(&state.active_database).unwrap()
                        .records.new_record(&table);

                    println!("{:?}", db_mgr.databases.get(&state.active_database).unwrap()
                        .records.records_by_table.get(&table_id));
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
                Instruction::SetField { field, value, repetition } => {

                    let context = EmulatorContext {
                        database_mgr: &*db_mgr,
                        variables: self.variables.last().unwrap(),
                        globals: &self.globals,
                        window_mgr: &*window_mgr,
                        state: &*state,
                    };
                    let val = value.eval(&context).unwrap();

                    let occurrence = db_mgr.databases.get(&state.active_database)
                        .unwrap()
                        .file.schema.relation_graph.nodes.iter()
                        .find(|occ| occ.id == field.table_occurrence_id)
                        .unwrap();

                    let source_table = occurrence.base.table_id;
                    let ds = occurrence.base.data_source;

                    if ds == 0 {
                        // Same data source as starting table
                        db_mgr.databases.get_mut(&state.active_database).unwrap()
                            .records.set_field(source_table, 1, field.field_id, val)
                    } else {
                        let source = db_mgr.databases.get(&state.active_database).unwrap()
                            .file.data_sources.iter()
                            .find(|source| source.id == ds)
                            .map(|source| source.name.clone())
                            .unwrap();

                        let other_handle = db_mgr.databases.get_mut(&source).unwrap();
                        other_handle.records.set_field(source_table, 1, field.field_id, val);
                    }
                    ip.0 += 1;
                },
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
