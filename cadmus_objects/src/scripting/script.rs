
use std::collections::HashMap;

use super::instructions::*;

use crate::{file::File, metadata::Metadata};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ScriptStep {
    pub id: u32,
    pub instruction: Instruction,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Script {
    pub id: u32,
    pub name: String,
    pub args: Vec<String>,
    pub instructions: Vec<ScriptStep>,
    pub metadata: Metadata,
}

impl Script {
    pub fn to_cad(&self, file: &File, externs: HashMap<usize, File>) -> String {
        let mut buffer = format!("script %{} {} = {{\n", self.id, self.name);

        let mut indent = 4;
        for instruction in &self.instructions {

            buffer.push_str(&" ".repeat(indent));

            buffer.push_str(&format!("{:?}", instruction.instruction));

            match instruction.instruction {
                Instruction::Loop | Instruction::If { .. } | Instruction::ElseIf { .. } => indent += 4,
                Instruction::EndIf | Instruction::EndLoop => indent -= 4,
                _ => {}
            }

        }

        buffer.push_str("\n\n");
        buffer
    }
}

