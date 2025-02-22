
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
                    let window = window_mgr.windows.get_mut(&state.active_window).unwrap();

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

                    let record_id = db_mgr.databases.get_mut(&state.active_database).unwrap()
                        .records.new_record(&table);

                    window.append_record_to_found_sets(record_id, table_id, db_mgr.databases.get(&state.active_database).unwrap());

                    ip.0 += 1;
                },
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
                },
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

                    let window = window_mgr.windows.get(&state.active_window).unwrap();
                    let current_set = window.found_sets
                        .iter()
                        .find(|set| set.table_occurrence_ref.table_occurrence_id == occurrence.id)
                        .unwrap();

                    if current_set.cursor.is_none() { eprintln!("Unable to set field, as there are no records present for this table."); ip.0 += 1; continue; }

                    if ds == 0 {
                        // Same data source as starting table
                        db_mgr.databases.get_mut(&state.active_database).unwrap()
                            .records.set_field(source_table, current_set.records[current_set.cursor.unwrap() as usize], field.field_id, val)
                    } else {
                        let source = db_mgr.databases.get(&state.active_database).unwrap()
                            .file.data_sources.iter()
                            .find(|source| source.id == ds)
                            .map(|source| source.name.clone())
                            .unwrap();

                        let other_handle = db_mgr.databases.get_mut(&source).unwrap();
                        other_handle.records.set_field(source_table, 1, field.field_id, val);
                    }
                    println!("{:?}", db_mgr.databases.get(&state.active_database).unwrap()
                        .records.records_by_table.get(&source_table));
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

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::path::Path;
    use std::fs::read_to_string;
    #[test]
    fn quotes_make_10_quotes() {
        let mut emulator = Emulator::new();
        let mut stored_tests = vec![];
        let code = read_to_string(Path::new("test_data/cad_files/multi_file_solution/quotes.cad")).unwrap();
        stored_tests.extend(crate::cadlang::compiler::compile_to_file(code).unwrap().tests);

        let to_run = stored_tests.iter()
            .find(|test| test.name == "make_10_quotes")
            .unwrap();

        emulator.run_test_with_file(&to_run, "test_data/cad_files/multi_file_solution/quotes.cad".to_string());

        let window = emulator.window_mgr.windows.get(&emulator.state.active_window).unwrap();
        assert_eq!(window.found_sets.len(), 5);
        let quotes_set = window.found_sets.iter().find(|set| set.table_occurrence_ref.table_occurrence_id == 1).unwrap();
        assert_eq!(quotes_set.records.len(), 10);
        let db = emulator.database_mgr.databases.get("test_data/cad_files/multi_file_solution/quotes.cad").unwrap();
        for x in 0..10 {
            assert_eq!(db.records.records_by_table.get(&1).unwrap()
                .iter()
                .find(|record| record.id == quotes_set.records[x as usize])
                .unwrap().fields[0].1, x.to_string())
        }
    }
}
