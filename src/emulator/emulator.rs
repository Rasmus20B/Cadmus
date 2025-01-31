
use std::collections::{HashMap, HashSet};
use color_print::cprintln;
use super::layout::Layout;
use crate::schema::*;
use super::window::Window;
use super::database::Database;
use super::layout_mgr::LayoutMgr;
use crate::fm_script_engine::fm_script_engine_instructions::Instruction;
use super::calc_engine::calc_eval::*;
use std::cell::RefCell;
use std::rc::Rc;

type window_id_t = u32;
type db_id_t = u32;
type tblocc_id_t = u32;
type layout_id_t = u32;

/* Emulator structure
 * ==================
 * The emulator will store duplicate state data that is 
 * contained within each of the structures. For example:
 *
 * A window will contain the layout it is currently using,
 * However we will also cache the current layout in the 
 * struct.
 *
 * The cached values will be updated whenever the ids of
 * the internal struct are changed.
 *
 * Calculation Interaction
 * =======================
 *
 * For example, referencing the field for an external data
 * source's record. Within the calculation we would keep
 * the state of where currently are, use that to access the
 * Global ID of the requested file, and the table information
 * needed, then use the related criteria to filter.
 * 
 */


pub struct IDCache {
    dirty: bool,
    active_window: window_id_t,
    active_layout: layout_id_t,
    active_occurrence: tblocc_id_t,
    active_database: db_id_t,
}

impl IDCache {
    pub fn new() -> Self {
        Self {
            dirty: false,
            active_window: 0,
            active_layout: 0,
            active_occurrence: 0,
            active_database: 0,
        }
    }
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub value: String,
    pub global: bool,
}

impl Variable {
    pub fn new(n: String, val: String, g: bool) -> Self {
        Self {
            name: n,
            value: val,
            global: g,
        }
    } 
}

#[derive(PartialEq, Debug)]
pub enum TestState {
    Pass,
    Fail
}

enum Mode {
    Browse,
    Find,
}

pub struct ScriptEngineState<'a> {
    pub instruction_ptr: Vec<(String, usize)>,
    pub call_stack: Vec<&'a Script>,
    pub variables: Vec<HashMap<String, Variable>>,
    pub active_script: Option<String>, 
    pub loop_scopes: Vec<usize>,
    pub test_state: TestState,
    pub punc_stack: Vec<Instruction>,
    pub branch_taken: bool,
    mode: Mode,
}

impl<'a> ScriptEngineState<'a> {
    pub fn new() -> Self {
        Self {
            instruction_ptr: vec![],
            call_stack: vec![],
            variables: vec![],
            active_script: None,
            loop_scopes: vec![],
            test_state: TestState::Pass,
            punc_stack: vec![],
            branch_taken: false,
            mode: Mode::Browse,
        }
    }
}

impl<'a> Default for ScriptEngineState<'a> {
    fn default() -> Self {
        Self {
            instruction_ptr: vec![],
            call_stack: vec![],
            variables: vec![],
            active_script: None,
            loop_scopes: vec![],
            test_state: TestState::Pass,
            punc_stack: vec![],
            branch_taken: false,
            mode: Mode::Browse,
        }
    }
}


pub struct Emulator<'a> {

    // Cached current DBObject IDs (layouts, windows)
    pub cache: IDCache,

    // Scripting State
    pub scr_state: ScriptEngineState<'a>,

    pub layout_mgr: LayoutMgr,

    pub windows: HashMap<window_id_t, Window>,
    pub layouts: HashMap<layout_id_t, Layout>,
    pub databases: HashMap<db_id_t, Database>,
    pub scripts: HashMap<String, Script>,

    // External Data Source Translations
    pub ext_db_lookup: HashMap<(db_id_t, db_id_t), db_id_t>,
}

impl<'a> Emulator<'a> {
    pub fn new() -> Self {
        Self {
            cache: IDCache::new(),
            scr_state: ScriptEngineState::new(),
            layout_mgr: LayoutMgr::new(),
            windows: HashMap::new(),
            layouts: HashMap::new(),
            databases: HashMap::new(),
            scripts: HashMap::new(),
            ext_db_lookup: HashMap::new(),
        }
    }

    pub fn load_schema(schema: &Schema) {
    }

    pub fn get_active_database(&self) -> &Database {
        &self.databases[&self.cache.active_database]
    }

    pub fn get_active_database_mut(&mut self) -> Option<&mut Database> {
        self.databases.get_mut(&self.cache.active_database)
    }

    pub fn step(&mut self) {
        let mut ip_handle: (String, usize);
        let n_stack = self.scr_state.instruction_ptr.len() - 1;
        ip_handle = self.scr_state.instruction_ptr[n_stack].clone();
        let s_name = self.scr_state.instruction_ptr[n_stack].0.clone();

        let script_handle = *self.scr_state.call_stack.last().unwrap();

        if script_handle.instructions.is_empty() 
            || ip_handle.1 > script_handle.instructions.len() - 1 {
                println!("Popping script: {}", ip_handle.0);
                self.scr_state.instruction_ptr.pop();
                return;
        }

        let mut cur_instruction = &script_handle.instructions[ip_handle.1];
        match &cur_instruction.opcode {
            Instruction::PerformScript => {
                let script_name = eval_calculation(&cur_instruction.switches[0], self);
                self.scr_state.variables.push(HashMap::new());

                for s in &self.scripts {
                    if s.1.name == script_name {
                        self.scr_state.instruction_ptr[n_stack].1 += 1;
                        self.scr_state.instruction_ptr.push((script_name.clone(), 0));
                        break;
                    }
                }
            },
            Instruction::GoToLayout => {
                let mut name : &str = &eval_calculation(&cur_instruction.switches[0], self);
                name = name
                    .strip_prefix('"').unwrap()
                    .strip_suffix('"').unwrap();

                let occurrence = self.layout_mgr.lookup(name.to_string());
                if let Some(occurrence_uw) = occurrence {
                    self.get_active_database_mut().unwrap().set_current_occurrence(occurrence_uw as u16);
                }
                self.scr_state.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::GoToRecordRequestPage => {
                let val = eval_calculation(&cur_instruction.switches[0], self);
                let mut exit = false;
                if cur_instruction.switches.len() > 1 {
                    let res = eval_calculation(&cur_instruction.switches[1], self);
                    if res != "false" || !res.is_empty() || res != "0" {
                        exit = true;
                    }
                }
            },
        //
        //        match self.mode {
        //            Mode::Browse => {
        //                let cache_pos = self.database.get_current_occurrence().record_ptr;
        //                if let Ok(n) = val.parse::<usize>() {
        //                    self.database.goto_record(n);
        //                } else {
        //                    match val.as_str() {
        //                        "\"previous\"" => { self.database.goto_previous_record(); }
        //                        "\"next\"" => { self.database.goto_next_record(); }
        //                        "\"first\"" => { self.database.goto_first_record(); }
        //                        "\"last\"" => { self.database.goto_last_record(); }
        //                        _ => {}
        //                    }
        //                }
        //                if exit && self.database.get_current_occurrence().record_ptr == cache_pos {
        //                    while cur_instruction.opcode != Instruction::EndLoop {
        //                        cur_instruction = &script_handle.instructions[ip_handle.1];
        //                        ip_handle.1 += 1;
        //                    }
        //                    self.instruction_ptr[n_stack].1 = ip_handle.1; 
        //                    return;
        //                }
        //            },
        //            Mode::Find => {
        //
        //            }
        //        }
        //        self.instruction_ptr[n_stack].1 += 1;
        //    },
        //    Instruction::EnterFindMode => {
        //        self.mode = Mode::Find;
        //        self.instruction_ptr[n_stack].1 += 1;
        //    },
        //    Instruction::UnsortRecords => {
        //        self.instruction_ptr[n_stack].1 += 1;
        //    },
        //    Instruction::PerformFind => {
        //        let mut records: HashSet<usize> = HashSet::new();
        //        for criteria in &self.find_criteria {
        //            let values = self.database
        //                .get_field_vals_for_current_table(
        //                    criteria.0
        //                        .split("::")
        //                        .collect::<Vec<&str>>()[1]
        //                    );
        //
        //            let ids = values.iter()
        //                .enumerate()
        //                .filter(|x| *x.1 == criteria.1)
        //                .map(|x| x.0)
        //                .collect::<Vec<usize>>();
        //            records.extend(ids);
        //        }
        //
        //        let mut records = Vec::from_iter(records);
        //        records.sort();
        //        self.database.update_found_set(&records);
        //        self.mode = Mode::Browse;
        //        self.find_criteria.clear();
        //        self.instruction_ptr[n_stack].1 += 1;
        //    },
        //    Instruction::ShowAllRecords => {
        //        self.database.reset_found_set();
        //        self.instruction_ptr[n_stack].1 += 1;
        //    },
        //    Instruction::SetVariable => {
        //        let name : &str = cur_instruction.switches[0].as_ref();
        //        let val : &str = &self.eval_calculation(&cur_instruction.switches[1]);
        //
        //        let tmp = Variable::new(name.to_string(), val.to_string(), false);
        //        let handle = &mut self.variables[n_stack].get_mut(name);
        //        if handle.is_none() {
        //            self.variables[n_stack].insert(name.to_string(), tmp);
        //        } else {
        //            handle.as_mut().unwrap().value = tmp.value;
        //        }
        //        self.instruction_ptr[n_stack].1 += 1;
        //    },
            Instruction::SetField => {
                let name : &str = cur_instruction.switches[0].as_ref();
                let val : &str = &mut eval_calculation(&cur_instruction.switches[1], self);
                let parts : Vec<&str> = name.split("::").collect();

                match self.scr_state.mode {
                    Mode::Browse => {
                        //*self.database.get_current_record_by_table_field_mut(parts[0], parts[1]).unwrap() = val.to_string();
                    },
                    Mode::Find => {
                        //self.find_criteria.push((name.to_string(), val.to_string()));
                    }
                }
                self.scr_state.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::Loop => {
                self.scr_state.loop_scopes.push(ip_handle.1);
                self.scr_state.punc_stack.push(Instruction::Loop);
                self.scr_state.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::If => {
                let val = eval_calculation(&cur_instruction.switches[0], self);
                if val == "true" {
                    self.scr_state.instruction_ptr[n_stack].1 += 1;
                    self.scr_state.branch_taken = true;
                } else {
                    while self.scr_state.instruction_ptr[n_stack].1 < script_handle.instructions.len() {
                        cur_instruction = &script_handle.instructions[self.scr_state.instruction_ptr[n_stack].1];
                        match cur_instruction.opcode {
                            Instruction::EndIf => {
                                return;
                            },
                            Instruction::Else => {
                                return;
                            },
                            Instruction::ElseIf => {
                                return;
                            },
                            _ => {}
                        }
                        self.scr_state.instruction_ptr[n_stack].1 += 1;
                    }
                }
            },
            Instruction::ElseIf => {
                if self.scr_state.branch_taken {
                    while self.scr_state.instruction_ptr[n_stack].1 < script_handle.instructions.len() {
                        cur_instruction = &script_handle.instructions[self.scr_state.instruction_ptr[n_stack].1];
                        match cur_instruction.opcode {
                            Instruction::EndIf => {
                                self.scr_state.branch_taken = false;
                                return;
                            },
                            _ => {self.scr_state.instruction_ptr[n_stack].1 += 1;}
                        }
                    }
                }
                let val = eval_calculation(&cur_instruction.switches[0], self);
                if val == "true" {
                    self.scr_state.instruction_ptr[n_stack].1 += 1;
                    self.scr_state.branch_taken = true;
                } else {
                    self.scr_state.instruction_ptr[n_stack].1 += 1;
                    while self.scr_state.instruction_ptr[n_stack].1 < script_handle.instructions.len() {
                        cur_instruction = &script_handle.instructions[self.scr_state.instruction_ptr[n_stack].1];
                        match cur_instruction.opcode {
                            Instruction::EndIf => {
                                return;
                            },
                            Instruction::Else => {
                                return;
                            },
                            Instruction::ElseIf => {
                                return;
                            },
                            _ => { self.scr_state.instruction_ptr[n_stack].1 += 1; }
                        }
                    }
                }
            }
            Instruction::Else => {
                if self.scr_state.branch_taken {
                    while self.scr_state.instruction_ptr[n_stack].1 < script_handle.instructions.len() {
                        cur_instruction = &script_handle.instructions[self.scr_state.instruction_ptr[n_stack].1];
                        match cur_instruction.opcode {
                            Instruction::EndIf => {
                                self.scr_state.branch_taken = false;
                                return;
                            },
                            _ => {self.scr_state.instruction_ptr[n_stack].1 += 1;}
                        }
                    }
                }
                self.scr_state.instruction_ptr[n_stack].1 += 1;
            }

            Instruction::EndIf => {
                self.scr_state.branch_taken = false;
                self.scr_state.instruction_ptr[n_stack].1 += 1;
            }
        //    Instruction::EndLoop => {
        //        if *self.punc_stack.last().unwrap_or(&Instruction::EndLoop) != Instruction::Loop {
        //            eprintln!("invalid scope resultion. Please check that loop and if blocks are terminated correctly.");
        //        }
        //        self.instruction_ptr[n_stack].1 = self.loop_scopes.last().unwrap() + 1;
        //    },
        //    Instruction::ExitLoopIf => {
        //        let val : &str = &self.eval_calculation(&cur_instruction.switches[0]);
        //        if val == "true" {
        //            while cur_instruction.opcode != Instruction::EndLoop {
        //                cur_instruction = &script_handle.instructions[ip_handle.1];
        //                ip_handle.1 += 1;
        //            }
        //            self.instruction_ptr[n_stack].1 = ip_handle.1; 
        //        } else {
        //            self.instruction_ptr[n_stack].1 += 1;
        //        }
        //    }
        //    Instruction::NewRecordRequest => {
        //        self.database.create_record();
        //        self.instruction_ptr[n_stack].1 += 1;
        //    },
        //    Instruction::ShowCustomDialog => {
        //        println!("{}", &self.eval_calculation(&cur_instruction.switches[0]));
        //        self.instruction_ptr[n_stack].1 += 1;
        //    },
        //    Instruction::Assert => {
        //        let val : &str = &self.eval_calculation(&cur_instruction.switches[0]);
        //        if val == "false" {
        //            cprintln!("<red>Assertion failed<red>: {}", cur_instruction.switches[0]);
        //            self.test_state = TestState::Fail;
        //        } 
        //        self.instruction_ptr[n_stack].1 += 1;
        //    },
        //    Instruction::CommentedOut | Instruction::BlankLineComment => {
        //        self.instruction_ptr[n_stack].1 += 1;
        //    }
            _ => {
                eprintln!("Unimplemented instruction: {:?}", cur_instruction.opcode);
                self.scr_state.instruction_ptr[n_stack].1 += 1;
            }

        }
    }
}






