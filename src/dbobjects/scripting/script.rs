
use super::instructions::*;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ScriptStep {
    pub opcode: Instruction,
    pub index: usize,
    pub switches: Vec<String>,
}

pub struct Script {
    pub script_name: String,
    pub instructions: Vec<Instruction>,
}

