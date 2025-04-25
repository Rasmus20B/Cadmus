
use crate::dbobjects::{calculation::CalculationString, scripting::{instructions::Instruction, script::Script}};

use super::proto_instruction::*;

#[derive(Debug, Clone)]
pub struct ProtoScript {
    pub name: String,
    pub instructions: Vec<ProtoInstruction>,
}
