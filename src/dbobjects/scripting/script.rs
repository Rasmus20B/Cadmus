
use super::instructions::*;

use crate::dbobjects::metadata::Metadata;

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

