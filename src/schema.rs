use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::{fm_script_engine::fm_script_engine_instructions::ScriptStep, hbam::fs::HBAMInterface};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum DBObjectKind {
    Table,
    TableOccurrence,
    Field,
    Relation,
    ValueList,
    Script,
    Layout,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum DBObjectStatus {
    Unmodified,
    Modified,
    Created,
    Deleted,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Table {
    pub id: usize,
    pub name: String,
    pub fields: HashMap<usize, Field>,
    pub created_by: String,
    pub modified_by: String,
}

impl Table {
    pub fn new(id_: usize) -> Self {
        Self {
            id: id_,
            name: String::new(),
            fields: HashMap::new(),
            created_by: String::new(),
            modified_by: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum SerialTrigger {
    OnCreation,
    OnCommit,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum AutoEntryDataPresets {
    Date,
    Time,
    Timestamp,
    Name,
    AccountName,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum AutoEntryType {
    NA,
    Serial { next: usize, increment: usize, trigger: SerialTrigger },
    Lookup { from: String, to: String },
    Creation(AutoEntryDataPresets),
    Modification(AutoEntryDataPresets),
    LastVisited,
    Data(String),
    Calculation{code: String, noreplace: bool},
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AutoEntry {
    pub nomodify: bool,
    pub definition: AutoEntryType
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ValidationTrigger {
    OnEntry,
    OnCommit,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ValidationType {
    NotEmpty,
    Unique,
    Required,
    MemberOf(String),
    Range{start: usize, end: usize},
    Calculation(String),
    MaxChars(usize),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Validation {
    pub trigger: ValidationTrigger,
    pub user_override: bool,
    pub checks: Vec<ValidationType>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Field {
    pub id: usize,
    pub name: String,
    pub created_by: String,
    pub modified_by: String,
    pub validation: Validation,
    pub autoentry: AutoEntry,
    pub global: bool,
    pub repetitions: usize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TableOccurrence {
    pub id: usize,
    pub name: String,
    pub table_actual: u16,
    pub table_actual_name: String,
    pub created_by: String,
    pub modified_by: String,
}

impl TableOccurrence {
    pub fn new(id_: usize) -> Self {
        Self {
            id: id_,
            name: String::new(),
            table_actual: 0,
            table_actual_name: String::new(),
            created_by: String::new(),
            modified_by: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum RelationComparison {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Cartesian
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct RelationCriteria {
    pub field1: u16,
    pub field2: u16,
    pub comparison: RelationComparison,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Relation {
    pub id: usize,
    pub table1: u16,
    pub table1_name: String,
    pub table1_data_source: u8,
    pub table2: u16,
    pub table2_name: String,
    pub table2_data_source: u8,
    pub criterias: Vec<RelationCriteria>,
}

impl Relation {
    pub fn new(id_: usize) -> Self {
        Self {
            id: id_,
            table1: 0,
            table1_name: String::new(),
            table1_data_source: 0,
            table2: 0,
            table2_name: String::new(),
            table2_data_source: 0,
            criterias: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ValueList {
    pub id: usize,
    pub name: String,
    pub created_by: String,
    pub modified_by: String,
}

impl ValueList {
    pub fn new(id_: usize) -> Self {
        Self {
            id: id_,
            name: String::new(),
            created_by: String::new(),
            modified_by: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Script {
    pub id: usize,
    pub name: String,
    pub instructions: Vec<ScriptStep>,
    pub arguments: Vec<String>,
    pub created_by: String,
    pub modified_by: String,
}

impl Script {
    pub fn new() -> Self {
        Self {
            id: 0,
            name: String::new(),
            instructions: vec![],
            arguments: vec![],
            created_by: String::new(),
            modified_by: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Test {
    pub id: usize,
    pub name: String,
    pub script: Script,
}

impl Test {
    pub fn new() -> Self {
        Self {
            id: 0,
            name: String::new(),
            script: Script::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LayoutFM {
    pub id: usize,
    pub name: String,
    pub table_occurrence: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Schema {
    pub tables: HashMap<usize, Table>,
    pub fields: HashMap<usize, Field>,
    pub table_occurrences: HashMap<usize, TableOccurrence>,
    pub relations: HashMap<usize, Relation>,
    pub value_lists: HashMap<usize, ValueList>,
    pub layouts: HashMap<usize, LayoutFM>,
    pub scripts: HashMap<usize, Script>,
    pub tests: HashMap<usize, Test>,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            fields: HashMap::new(),
            table_occurrences: HashMap::new(),
            relations: HashMap::new(),
            value_lists: HashMap::new(),
            layouts: HashMap::new(),
            scripts: HashMap::new(),
            tests: HashMap::new(),
        }
    }
}

impl From<&mut HBAMInterface> for Schema {
    fn from(hbam: &mut HBAMInterface) -> Schema {
        let mut result = Schema::new();
        result.tables.extend(hbam.get_tables());
        hbam.get_table_occurrences(&mut result);
        hbam.get_fields(&mut result).expect("Unable to get fields from hbam file.");
        hbam.get_layouts(&mut result).expect("Unable to get fields from hbam file.");
        result
    }
}
