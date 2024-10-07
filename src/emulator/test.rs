use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::write;
use std::ops::Deref;

use chrono::Local;
use clap::Parser;
use color_print::cprintln;
use crate::schema::Script;
use crate::schema::Test;
use crate::schema;
use crate::fm_script_engine::fm_script_engine_instructions::Instruction;
use crate::fm_script_engine::fm_script_engine_instructions::ScriptStep;
use super::calc_lexer;
use super::{calc_parser, database::*};

use super::calc_parser::Node;
use super::calc_tokens;
use super::calc_tokens::TokenType;
use super::database::Database;
use super::layout_mgr::LayoutMgr;

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

#[derive(Debug)]
enum EvaluateError {
    DivideByZero { operand_left: String, operand_right: String },
    InvalidOperation { operand_left: String, operator: String, operand_right: String },
    InvalidArgument { function: String, argument: String },
    UnimplementedFunction { function: String },
}

pub struct VMTable {
    pub name: String,
    pub records: HashMap<String, Vec<String>>,
}

#[derive(PartialEq, Debug)]
pub enum TestState {
    Pass,
    Fail
}

#[derive(PartialEq, Debug)]
enum OperandType {
    Number,
    Text,
    FieldName,
}

#[derive(PartialEq, Debug)]
struct Operand<'a> {
    value: &'a str,
    otype: OperandType
}

type Record = usize;

enum Mode {
    Browse,
    Find,
}

pub struct TestEnvironment<'a> {
    pub file_handle: &'a schema::Schema, // TODO: Doesn't need to be stored here

    /* Each table has it's own record pointer, as per FileMaker */
    /* Each script has it's own instruction ptr.
     * on calling of a script, a new ptr is pushed.
     * When script is finished, we pop the instruction ptr.
     * Nothing more complex than function calls. No generators etc,
     * so this is fine.
     */

    /* Language interpretation */
    pub instruction_ptr: Vec<(String, usize)>,
    pub variables: Vec<HashMap<String, Variable>>,
    pub current_test: Option<Test>, 
    pub loop_scopes: Vec<usize>,
    pub test_state: TestState,
    pub punc_stack: Vec<Instruction>,
    pub branch_taken: bool,
    mode: Mode,

    /* Data storage and behaviour */
    pub database: Database,
    pub layout_mgr: LayoutMgr,
    pub find_criteria: Vec<(String, String)>,
}
impl<'a> TestEnvironment<'a> {
    pub fn new(file: &'a schema::Schema) -> Self {
        Self {
            file_handle: file,
            instruction_ptr: vec![],
            variables: vec![],
            current_test: None,
            loop_scopes: vec![],
            test_state: TestState::Pass,
            punc_stack: vec![],
            branch_taken: false,
            layout_mgr: LayoutMgr::new(),
            database: Database::new(),
            find_criteria: vec![],
            mode: Mode::Browse,
        }
    }

    pub fn generate_database(&mut self) {
        self.database.generate_from_fmp12(&self.file_handle);
    }

    pub fn generate_layout_mgr(&mut self) {
        for (_, layout) in &self.file_handle.layouts {
            self.layout_mgr.add_mapping(layout.name.clone(), layout.table_occurrence);
        }
    }

    pub fn generate_test_environment(&mut self) {
        self.generate_database();
        self.generate_layout_mgr();
    }

    #[allow(unused)]
    pub fn run_tests(&mut self) {
        for (_, test) in &self.file_handle.tests {
            /* 1. Run the script 
             * 2. Check Assertions defined in test component
             * 3. Clean the test environment for next test */
            println!("Running test: {}", test.name);
            self.load_test(test.clone());
            while self.test_state == TestState::Pass && !self.instruction_ptr.is_empty() {
                self.step();
            }
            if self.test_state == TestState::Pass {
                cprintln!("Test {} outcome: <green>Success</green>", self.current_test.as_ref().unwrap().name);
            } else if self.test_state == TestState::Fail {
                cprintln!("Test {} outcome: <red>Fail</red>", self.current_test.as_ref().unwrap().name);
            }
        }
    }
    pub fn run_tests_with_cleanup(&mut self) {
        for (_, test) in &self.file_handle.tests {
            /* 1. Run the script 
             * 2. Check Assertions defined in test component
             * 3. Clean the test environment for next test */
            println!("Running test: {}", test.name);
            self.load_test(test.clone());
            while self.test_state == TestState::Pass && !self.instruction_ptr.is_empty() {
                self.step();
            }
            if self.test_state == TestState::Pass {
                cprintln!("Test {} outcome: <green>Success</green>", self.current_test.as_ref().unwrap().name);
            } else if self.test_state == TestState::Fail {
                cprintln!("Test {} outcome: <red>Fail</red>", self.current_test.as_ref().unwrap().name);
            }
        }

        self.database.clear_records();
    }

    pub fn load_test(&mut self, test: Test) {
        self.current_test = Some(test.clone());
        let script_name = &self.current_test.as_ref().unwrap().script.name;
        self.instruction_ptr.push((script_name.to_string(), 0));
        self.variables.push(HashMap::new());
    }

    pub fn step(&mut self) {
        assert!(self.current_test.is_some());
        let mut ip_handle: (String, usize);
        let n_stack = self.instruction_ptr.len() - 1;
        ip_handle = self.instruction_ptr[n_stack].clone();
        let s_name = self.instruction_ptr[n_stack].0.clone();

        let script_handle = if self.instruction_ptr.len() > 1 {
            self.file_handle.scripts.iter()
                .map(|script| script.1)
                .find(|script| script.name == s_name)
        } else {
            Some(&self.current_test.as_ref().unwrap().script)
        };

        let script_handle = script_handle.expect("Unable to find script.");

        if script_handle.instructions.is_empty() 
            || ip_handle.1 > script_handle.instructions.len() - 1 {
                println!("Popping script: {}", ip_handle.0);
                self.instruction_ptr.pop();
                return;
        }

        let mut cur_instruction = &script_handle.instructions[ip_handle.1];
        // println!("instr: {:?}", cur_instruction);
        match &cur_instruction.opcode {
            Instruction::PerformScript => {
                let script_name = self.eval_calculation(&cur_instruction.switches[0])
                    .strip_suffix('"').unwrap()
                    .strip_prefix('"').unwrap().to_string();
                self.variables.push(HashMap::new());

                for s in &self.file_handle.scripts {
                    if s.1.name == script_name {
                        self.instruction_ptr[n_stack].1 += 1;
                        self.instruction_ptr.push((script_name.clone(), 0));
                        println!("calling {}", script_name);
                        break;
                    }
                }
            },
            Instruction::GoToLayout => {
                let mut name : &str = &self.eval_calculation(&cur_instruction.switches[0]);
                name = name
                    .strip_prefix('"').unwrap()
                    .strip_suffix('"').unwrap();

                let occurrence = self.layout_mgr.lookup(name.to_string());
                if occurrence.is_some() {
                    self.database.set_current_occurrence(occurrence.unwrap() as u16);
                }
                self.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::GoToRecordRequestPage => {
                let val = &self.eval_calculation(&cur_instruction.switches[0]);
                let mut exit = false;
                if cur_instruction.switches.len() > 1 {
                    let res = &self.eval_calculation(&cur_instruction.switches[1]);
                    if res != "false" || res != "" || res != "0" {
                        exit = true;
                    }
                }

                match self.mode {
                    Mode::Browse => {
                        let cache_pos = self.database.get_current_occurrence().record_ptr;
                        if let Ok(n) = val.parse::<usize>() {
                            self.database.goto_record(n);
                        } else {
                            match val.as_str() {
                                "\"previous\"" => { self.database.goto_previous_record(); }
                                "\"next\"" => { self.database.goto_next_record(); }
                                "\"first\"" => { self.database.goto_first_record(); }
                                "\"last\"" => { self.database.goto_last_record(); }
                                _ => {}
                            }
                        }
                        if exit && self.database.get_current_occurrence().record_ptr == cache_pos {
                            while cur_instruction.opcode != Instruction::EndLoop {
                                cur_instruction = &script_handle.instructions[ip_handle.1];
                                ip_handle.1 += 1;
                            }
                            self.instruction_ptr[n_stack].1 = ip_handle.1; 
                            return;
                        }
                    },
                    Mode::Find => {

                    }
                }
                self.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::EnterFindMode => {
                self.mode = Mode::Find;
                self.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::UnsortRecords => {
                self.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::PerformFind => {
                let mut records: HashSet<usize> = HashSet::new();
                for criteria in &self.find_criteria {
                    let values = self.database
                        .get_field_vals_for_current_table(
                            &criteria.0
                                .split("::")
                                .collect::<Vec<&str>>()[1]
                            );

                    let ids = values.into_iter()
                        .enumerate()
                        .filter(|x| *x.1 == criteria.1)
                        .map(|x| x.0)
                        .collect::<Vec<usize>>();
                    records.extend(ids);
                }

                let mut records = Vec::from_iter(records);
                records.sort();
                self.database.update_found_set(&records);
                self.mode = Mode::Browse;
                self.find_criteria.clear();
                self.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::ShowAllRecords => {
                self.database.reset_found_set();
                self.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::SetVariable => {
                let name : &str = cur_instruction.switches[0].as_ref();
                let val : &str = &self.eval_calculation(&cur_instruction.switches[1]);

                let tmp = Variable::new(name.to_string(), val.to_string(), false);
                let handle = &mut self.variables[n_stack].get_mut(name);
                if handle.is_none() {
                    self.variables[n_stack].insert(name.to_string(), tmp);
                } else {
                    handle.as_mut().unwrap().value = tmp.value;
                }
                self.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::SetField => {
                let name : &str = cur_instruction.switches[0].as_ref();
                let val : &str = &mut self.eval_calculation(&cur_instruction.switches[1]);
                let parts : Vec<&str> = name.split("::").collect();

                match self.mode {
                    Mode::Browse => {
                        *self.database.get_current_record_by_table_field_mut(parts[0], parts[1]).unwrap() = val.to_string();
                    },
                    Mode::Find => {
                        self.find_criteria.push((name.to_string(), val.to_string()));
                    }
                }
                self.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::Loop => {
                self.loop_scopes.push(ip_handle.1);
                self.punc_stack.push(Instruction::Loop);
                self.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::If => {
                let val = &self.eval_calculation(&cur_instruction.switches[0]);
                if val == "true" {
                    self.instruction_ptr[n_stack].1 += 1;
                    self.branch_taken = true;
                } else {
                    while self.instruction_ptr[n_stack].1 < script_handle.instructions.len() {
                        cur_instruction = &script_handle.instructions[self.instruction_ptr[n_stack].1];
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
                        self.instruction_ptr[n_stack].1 += 1;
                    }
                }
            },
            Instruction::ElseIf => {
                if self.branch_taken == true {
                    while self.instruction_ptr[n_stack].1 < script_handle.instructions.len() {
                        cur_instruction = &script_handle.instructions[self.instruction_ptr[n_stack].1];
                        match cur_instruction.opcode {
                            Instruction::EndIf => {
                                self.branch_taken = false;
                                return;
                            },
                            _ => {self.instruction_ptr[n_stack].1 += 1;}
                        }
                    }
                }
                let val = &self.eval_calculation(&cur_instruction.switches[0]);
                if val == "true" {
                    self.instruction_ptr[n_stack].1 += 1;
                    self.branch_taken = true;
                } else {
                    self.instruction_ptr[n_stack].1 += 1;
                    while self.instruction_ptr[n_stack].1 < script_handle.instructions.len() {
                        cur_instruction = &script_handle.instructions[self.instruction_ptr[n_stack].1];
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
                            _ => { self.instruction_ptr[n_stack].1 += 1; }
                        }
                    }
                }
            }
            Instruction::Else => {
                if self.branch_taken == true {
                    while self.instruction_ptr[n_stack].1 < script_handle.instructions.len() {
                        cur_instruction = &script_handle.instructions[self.instruction_ptr[n_stack].1];
                        match cur_instruction.opcode {
                            Instruction::EndIf => {
                                self.branch_taken = false;
                                return;
                            },
                            _ => {self.instruction_ptr[n_stack].1 += 1;}
                        }
                    }
                }
                self.instruction_ptr[n_stack].1 += 1;
            }

            Instruction::EndIf => {
                self.branch_taken = false;
                self.instruction_ptr[n_stack].1 += 1;
            }
            Instruction::EndLoop => {
                if *self.punc_stack.last().unwrap_or(&Instruction::EndLoop) != Instruction::Loop {
                    eprintln!("invalid scope resultion. Please check that loop and if blocks are terminated correctly.");
                }
                self.instruction_ptr[n_stack].1 = self.loop_scopes.last().unwrap() + 1;
            },
            Instruction::ExitLoopIf => {
                let val : &str = &self.eval_calculation(&cur_instruction.switches[0]);
                if val == "true" {
                    while cur_instruction.opcode != Instruction::EndLoop {
                        cur_instruction = &script_handle.instructions[ip_handle.1];
                        ip_handle.1 += 1;
                    }
                    self.instruction_ptr[n_stack].1 = ip_handle.1; 
                } else {
                    self.instruction_ptr[n_stack].1 += 1;
                }
            }
            Instruction::NewRecordRequest => {
                self.database.create_record();
                self.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::ShowCustomDialog => {
                println!("{}", &self.eval_calculation(&cur_instruction.switches[0]));
                self.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::Assert => {
                let val : &str = &self.eval_calculation(&cur_instruction.switches[0]);
                if val == "false" {
                    cprintln!("<red>Assertion failed<red>: {}", cur_instruction.switches[0]);
                    self.test_state = TestState::Fail;
                } 
                self.instruction_ptr[n_stack].1 += 1;
            },
            Instruction::CommentedOut | Instruction::BlankLineComment => {
                self.instruction_ptr[n_stack].1 += 1;
            }
            _ => {
                eprintln!("Unimplemented instruction: {:?}", cur_instruction.opcode);
                self.instruction_ptr[n_stack].1 += 1;
            }

        }
    }

    pub fn get_variable_by_name(&self, scope: usize, var: &str) -> Option<&Variable> {
        return self.variables[scope].get(var)
    }

    pub fn eval_calculation(&self, calculation: &str) -> String {
        let tokens = calc_lexer::lex(calculation).expect("Unable to lex calculation.");
        let ast = calc_parser::Parser::new(tokens).parse().expect("unable to parse tokens.");
        self.evaluate(ast).expect("Unable to evaluate expression.")
    }

    fn get_operand_val(&'a self, val: &'a str) -> Result<Operand<'a>, String> {
        let r = val.parse::<f64>();
        if r.is_ok() {
            return Ok(Operand {
                otype: OperandType::Number,
                value: val
            })
        }

        if val.starts_with("\"") && val.ends_with("\"") {
            return Ok(Operand {
                otype: OperandType::Text,
                value: val
            })
        }

        let fieldname = val.split("::").collect::<Vec<&str>>();
        if fieldname.len() == 2 {
            let val = self.database.get_found_set_record_field(fieldname[1]);
            return self.get_operand_val(val);
        } else {
            let scope = self.instruction_ptr.len() - 1;
            let var_val = self.variables[scope]
                .get(val);

            if var_val.is_none() {
                return Err("Unable to evaluate script step argument.".to_string());
            }
            return self.get_operand_val(&var_val.unwrap().value);
        }

    }


    pub fn evaluate(&self, ast: Box<Node>) -> Result<String, EvaluateError> {

        match *ast {
            Node::Unary { value, child } => {
                let val = self.get_operand_val(value.as_str()).unwrap()
                    .value.to_string();

                if child.is_none() {
                    return Ok(val);
                } 
                Ok(val.to_string())
            },
            Node::Grouping { ref left, operation, right } => {
                let lhs_wrap = &self.evaluate(left.clone())?;
                let rhs_wrap = &self.evaluate(right)?;

                let mut lhs = match *left.clone() {
                    Node::Number(val) => Operand { value: &val.to_string(), otype: OperandType::Number },
                    _ => self.get_operand_val(lhs_wrap).unwrap()
                };
                let mut rhs = self.get_operand_val(rhs_wrap).unwrap();

                match operation {
                    TokenType::Multiply => {
                        Ok((lhs.value.parse::<f32>().unwrap() * rhs.value.parse::<f32>().unwrap()).to_string())
                    }
                    TokenType::Plus => {
                        Ok((lhs.value.parse::<f32>().unwrap() + rhs.value.parse::<f32>().unwrap()).to_string())
                    }
                    TokenType::Ampersand => {
                        if lhs.otype == OperandType::Text {
                            lhs.value = lhs.value
                            .strip_prefix('"').unwrap()
                            .strip_suffix('"').unwrap();
                        }
                        if rhs.otype == OperandType::Text {
                            rhs.value = rhs.value
                            .strip_prefix('"').unwrap()
                            .strip_suffix('"').unwrap();
                        }
                        Ok(("\"".to_owned() + lhs.value + rhs.value + "\"").to_string())
                    }
                    _ => Ok(format!("Invalid operation {:?}.", operation).to_string())
                }
            }
            Node::Call { name, args } => {
                match name.as_str() {
                    "Abs" => { Ok(self.evaluate(args[0].clone())?
                        .parse::<f32>().expect("unable to perform Abs() on non-numeric")
                        .abs().to_string())
                    }
                    "Acos" => { Ok(self.evaluate(args[0].clone())?.parse::<f64>().unwrap().acos().to_string()) }
                    "Asin" => { Ok(std::cmp::min(self.evaluate(args[0].clone())?, self.evaluate(args[1].clone())?))}
                    "Char" => { 
                        let mut res = String::new();
                        res.push('\"');
                        let c = char::from_u32(self.evaluate(args[0].clone()).expect("Unable to evalue argument.").parse::<u32>().unwrap()).unwrap();
                        res.push(c);
                        res.push('\"');
                        Ok(res)
                    }
                    "Min" => { 
                        Ok(args
                            .into_iter()
                            .map(|x| {
                                self.evaluate(x.clone()).expect("Unable to evaluate argument.")
                            })
                            .min_by(|x, y| {
                                let lhs = self.get_operand_val(&x).unwrap();
                                let rhs = self.get_operand_val(&y).unwrap();

                                if lhs.otype == OperandType::Number && rhs.otype == OperandType::Number {
                                        lhs.value.parse::<f64>().expect("lhs is not a valid number")
                                            .partial_cmp(
                                        &rhs.value.parse::<f64>().expect("rhs is not a number.")
                                        ).unwrap()
                                } else {
                                    lhs.value.to_string().cmp(&rhs.value.to_string())
                                }
                            }).unwrap())
                    },
                    "Get" => {
                        match *args[0] {
                            Node::Unary { ref value, child: _ } => {
                                match value.as_str() {
                                    "AccountName" => Ok("\"Admin\"".to_string()),
                                    "CurrentTime" => Ok(Local::now().timestamp().to_string()),
                                    _ => { Err(EvaluateError::InvalidArgument { function: name, argument: value.to_string() }) }
                                }
                            }
                            // TODO: Evaluate expression into string and then do the match.
                            _ => unimplemented!()
                        }
                    }
                    _ => { Err(EvaluateError::UnimplementedFunction { function: name }) }
                }
            },
            Node::Binary { left, operation, right } => {
                let lhs_wrap = &self.evaluate(left).unwrap();
                let rhs_wrap = &self.evaluate(right).unwrap();
                let mut lhs = self.get_operand_val(lhs_wrap).unwrap();
                let mut rhs = self.get_operand_val(rhs_wrap).unwrap();

                match operation {
                    calc_tokens::TokenType::Plus => { 
                        if lhs.otype == OperandType::Text { lhs.value = "0" }
                        if rhs.otype == OperandType::Text { rhs.value = "0" }
                        Ok((lhs.value.parse::<f64>().unwrap()
                         + 
                         rhs.value.parse::<f64>().unwrap()
                         ).to_string())
                    },
                    calc_tokens::TokenType::Minus => { 
                        if lhs.otype == OperandType::Text { lhs.value = "0" }
                        if rhs.otype == OperandType::Text { rhs.value = "0" }
                        Ok((lhs.value.parse::<f64>().unwrap()
                         - 
                         rhs.value.parse::<f64>().unwrap()
                         ).to_string())
                    },
                    calc_tokens::TokenType::Multiply => { 
                        if lhs.otype == OperandType::Text { lhs.value = "0" }
                        if rhs.otype == OperandType::Text { rhs.value = "0" }
                        Ok((lhs.value.parse::<f64>().unwrap()
                         * 
                         rhs.value.parse::<f64>().unwrap()
                         ).to_string())
                    },
                    calc_tokens::TokenType::Divide => { 
                        if lhs.otype == OperandType::Text { lhs.value = "0" }
                        if rhs.otype == OperandType::Text { rhs.value = "0" }

                        let checked_rhs = rhs.value.parse::<f64>().unwrap();
                        if checked_rhs == 0.0 { 
                            return Err(EvaluateError::DivideByZero { 
                                operand_left: lhs.value.to_string(), 
                                operand_right: checked_rhs.to_string() 
                            }) 
                        }
                        Ok((lhs.value.parse::<f64>().unwrap()
                         / 
                         rhs.value.parse::<f64>().unwrap()
                         ).to_string())
                    },
                    _ => { unreachable!()}
                }
            },
            Node::Comparison { ref left, operation, ref right } => {
                Ok(match operation {
                    TokenType::Eq => { (self.evaluate(left.clone())? == self.evaluate(right.clone())?).to_string() },
                    TokenType::Neq => { (self.evaluate(left.clone())? != self.evaluate(right.clone())?).to_string() },
                    TokenType::Gt => { (self.evaluate(left.clone())? > self.evaluate(right.clone())?).to_string() },
                    TokenType::Gtq => { (self.evaluate(left.clone())? >= self.evaluate(right.clone())?).to_string() },
                    TokenType::Lt => { (self.evaluate(left.clone())? < self.evaluate(right.clone())?).to_string() },
                    TokenType::Ltq => { (self.evaluate(left.clone())? <= self.evaluate(right.clone())?).to_string() },
                    _ => unreachable!()
                })
            }
            Node::Concatenation { left, right } => { 
                let lhs = self.evaluate(left.clone())?;
                let rhs = self.evaluate(right.clone())?;
                let lhs = lhs.replace('"', "");
                let rhs = rhs.replace('"', "");
                Ok(format!("\"{lhs}{rhs}\""))
            }
            Node::Number(val) => Ok(val.to_string()),
            Node::Variable(val) => Ok(self.get_operand_val(&val).unwrap().value.to_string()),
            Node::Field(val) => Ok(self.get_operand_val(&val).unwrap().value.to_string()),
            Node::StringLiteral(val) => Ok(val.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::emulator::test::TestState;
    use crate::hbam::fs::HBAMInterface;
    use crate::{compile::compiler::compile_burn, schema::Schema};
    use super::TestEnvironment;

    #[test]
    pub fn prec_test() {
        let mut file = Schema::new();
        let code = "
        test basicTest:
            script: [
                define prec_test() {
                    set_variable(x, 2 + 3 * 4);
                    show_custom_dialog(x);
                    assert(x == 14);
                    set_variable(x, (2 + 3) * 4);
                    show_custom_dialog(x);
                    assert(x == 20);
                    set_variable(y, 6 / 2 * 3);
                    show_custom_dialog(y);
                    assert(y == 9);
                    set_variable(x, 10 - 2 + 3 * 4);
                    show_custom_dialog(x);
                    assert(x == 20);
                    set_variable(x, 2 * (3 + (4 - 1)));
                    show_custom_dialog(x);
                    assert(x == 12);
                    set_variable(x, 2 + 3 * (4 - 1) / 2);
                    show_custom_dialog(x);
                    assert(x == 6.5);
                }
            ]
        end test;
        ";

        let test = compile_burn(code);
        file.tests.extend(&mut test.tests.into_iter());
        let mut te : TestEnvironment = TestEnvironment::new(&file);
        te.generate_test_environment();
        te.run_tests();
        assert_eq!(te.test_state, TestState::Pass);
    }

    #[test]
    pub fn calc_test() {
        let mut file = Schema::new();
        let code = "
        test basicTest:
            script: [
                define paren_test() {
                    set_variable(x, 2 + 3 * 4);
                    show_custom_dialog(x);
                    assert(x == 14);
                    set_variable(x, (2 + 3) * 4);
                    show_custom_dialog(x);
                    assert(x == 20);
                    set_variable(x, (2 * (2 + 3)) & \" things\");
                    show_custom_dialog(x);
                    assert(x == \"10 things\");
                    set_variable(x, 2 * (2 * (2)));
                    show_custom_dialog(x);
                    assert(x == 8);
                    set_variable(y, 2 * (x * (2)));
                    show_custom_dialog(y);
                    assert(y == 32);
                    set_variable(x, 2 * Min((2 + 2 + 2), y));
                    show_custom_dialog(x);
                    assert(x == 12);
                }
            ]
        end test;
        ";

        let test = compile_burn(code);
        file.tests.extend(&mut test.tests.into_iter());
        let mut te : TestEnvironment = TestEnvironment::new(&file);
        te.generate_test_environment();
        te.run_tests();
        assert_eq!(te.test_state, TestState::Pass);
    }
    #[test]
    pub fn basic_loop_test() {
        let code = "
        test BasicTest:
          script: [
            define blank_test() {
              set_variable(x, 0);
              go_to_layout(\"second_table\");
              new_record_request();
              set_field(second_table::PrimaryKey, \"Kevin\");
              set_field(second_table::add, \"secret\");
              go_to_layout(\"blank\");
              loop {
                exit_loop_if(x == 10);
                new_record_request();
                assert(x != 10);
                if(x == 7) {
                    set_field(blank::PrimaryKey, \"Kevin\");
                } elif(x == 1) {
                    set_field(blank::PrimaryKey, \"alvin\" & \" Presley\");
                } elif(x == 2) {
                    set_field(blank::PrimaryKey, \"NAHHH\");
                    assert(blank::PrimaryKey == \"NAHHH\");
                } else {
                    set_field(blank::PrimaryKey, \"Jeff\" & \" Keighly\");
                }
                set_variable(x, x + 1);
              }
              enter_find_mode();
              set_field(blank::PrimaryKey, \"Kevin\");
              perform_find();
              assert(blank::PrimaryKey == \"Kevin\");

              enter_find_mode();
              set_field(blank::PrimaryKey, \"Jeff Keighly\");
              perform_find();
              go_to_record(\"first\");
              loop {
                assert(blank::PrimaryKey == \"Jeff Keighly\");
                go_to_record(\"next\", \"true\");
              }
              show_all_records();
              go_to_record(\"first\");
              loop {
                show_custom_dialog(blank::PrimaryKey);
                go_to_record(\"next\", \"true\");
              }
            }
          ],
        end test;";
        let input = Path::new("test_data/input/mixed.fmp12");
        let mut file = Schema::from(&mut HBAMInterface::new(&input));
        let tests = compile_burn(code);
        file.tests.extend(&mut tests.tests.into_iter());
        let mut te : TestEnvironment = TestEnvironment::new(&file);
        te.generate_test_environment();
        te.run_tests();
        assert_eq!(te.database.get_table("blank").unwrap().fields[0].records.len(), 10);
        assert_eq!(te.database.get_table("second_table").unwrap().fields[0].records.len(), 1);
        assert_eq!(te.database.get_record_by_field("PrimaryKey", 0).unwrap(), "\"Jeff Keighly\"");
        assert_eq!(te.database.get_record_by_field("PrimaryKey", 1).unwrap(), "\"alvin Presley\"");
        assert_eq!(te.database.get_record_by_field("PrimaryKey", 2).unwrap(), "\"NAHHH\"");
        assert_eq!(te.database.get_record_by_field("PrimaryKey", 3).unwrap(), "\"Jeff Keighly\"");
        assert_eq!(te.database.get_record_by_field("PrimaryKey", 4).unwrap(), "\"Jeff Keighly\"");
        assert_eq!(te.database.get_record_by_field("PrimaryKey", 5).unwrap(), "\"Jeff Keighly\"");
        assert_eq!(te.database.get_record_by_field("PrimaryKey", 6).unwrap(), "\"Jeff Keighly\"");
        assert_eq!(te.database.get_record_by_field("PrimaryKey", 7).unwrap(), "\"Kevin\"");
        assert_eq!(te.database.get_record_by_field("PrimaryKey", 8).unwrap(), "\"Jeff Keighly\"");
        assert_eq!(te.database.get_record_by_field("PrimaryKey", 9).unwrap(), "\"Jeff Keighly\"");
        te.database.goto_record(7);
        assert_eq!(te.database.get_current_record_field("PrimaryKey"), "\"Kevin\"");
        assert_eq!(te.database.get_occurrence_by_name("second_table").found_set.len(), 1);
        assert_eq!(te.database.get_related_record_field("second_table", "add").unwrap(), "\"secret\"");
        assert_eq!(te.test_state, TestState::Pass);
    }
}

