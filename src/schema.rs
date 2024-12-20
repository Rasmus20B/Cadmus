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
pub struct DBObjectReference {
    pub data_source: u16,
    pub top_id: u16,
    pub inner_id: u16,
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
pub enum DataType {
    Text,
    Number,
    Time,
    Date,
    Timestamp,
    Container,
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
    Lookup { from: DBObjectReference, to: DBObjectReference },
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
    pub dtype: DataType,
    pub validation: Validation,
    pub autoentry: AutoEntry,
    pub global: bool,
    pub repetitions: usize,
    pub created_by: String,
    pub modified_by: String,
}

impl Field {

    pub fn new(id_: usize, name_: String) -> Self {
        Self {
            id: id_,
            name: name_,
            created_by: String::from("admin"),
            modified_by: String::from("admin"),
            dtype: DataType::Text,
            global: false,
            repetitions: 1,
            autoentry: AutoEntry {
                definition: AutoEntryType::NA,
                nomodify: false,
            },
            validation: Validation {
                checks: vec![],
                message: String::new(),
                trigger: ValidationTrigger::OnEntry,
                user_override: true,
            },
        }
    }

    pub fn datatype(mut self, dtype_: DataType) -> Self {
        self.dtype = dtype_;
        self
    }

    pub fn repetitions(mut self, repetitions_: usize) -> Self {
        self.repetitions = repetitions_;
        self
    }

    pub fn created_by(mut self, account: String) -> Self {
        self.created_by = account;
        self
    }

    pub fn modified_by(mut self, account: String) -> Self {
        self.modified_by = account;
        self
    }

    pub fn autoentry(mut self, definition_: AutoEntryType, nomodify_: bool) -> Self {
        self.autoentry = AutoEntry {
            definition: definition_,
            nomodify: nomodify_,
        };
        self
    }

    pub fn validation(mut self, validation_: Validation) -> Self {
        self.validation = validation_;
        self
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TableOccurrence {
    pub id: usize,
    pub name: String,
    pub base_table: DBObjectReference,
    pub created_by: String,
    pub modified_by: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
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
pub enum RelationCriteria {
    ById { field1: u16, field2: u16, comparison: RelationComparison },
    ByName { field1: String, field2: String, comparison: RelationComparison },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Relation {
    pub id: usize,
    pub table1: DBObjectReference,
    pub table2: DBObjectReference,
    pub criterias: Vec<RelationCriteria>,
}

impl Relation {
    pub fn new(id_: usize, table1_: DBObjectReference, table2_: DBObjectReference) -> Self {
        Self {
            id: id_,
            table1: table1_,
            table2: table2_,
            criterias: vec![],
        }
    }

    pub fn criteria(mut self, criteria: RelationCriteria) -> Self {
        self.criterias.push(criteria);
        self
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ValueListSortBy {
    FirstField,
    SecondField,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ValueListDefinition {
    CustomValues(Vec<String>),
    FromField { field1: String, 
        field2: Option<String>, 
        from: Option<String>, 
        sort: ValueListSortBy, 
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ValueList {
    pub id: usize,
    pub name: String,
    pub definition: ValueListDefinition,
    pub created_by: String,
    pub modified_by: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Script {
    pub id: u16,
    pub name: String,
    pub instructions: Vec<ScriptStep>,
    pub arguments: Vec<String>,
    pub created_by: String,
    pub modified_by: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Test {
    pub id: u16,
    pub name: String,
    pub script: Script,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LayoutFMAttribute {

}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LayoutFM {
    pub id: usize,
    pub name: String,
    pub table_occurrence: DBObjectReference,
    pub table_occurrence_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DataSourceType {
    FileMaker,
    ODBC,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DataSource {
    pub id: usize,
    pub name: String,
    pub dstype: DataSourceType,
    pub filename: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Schema {
    pub tables: HashMap<usize, Table>,
    pub fields: HashMap<usize, Field>,
    pub table_occurrences: HashMap<usize, TableOccurrence>,
    pub relations: HashMap<usize, Relation>,
    pub value_lists: HashMap<usize, ValueList>,
    pub layouts: HashMap<usize, LayoutFM>,
    pub scripts: HashMap<u16, Script>,
    pub tests: HashMap<u16, Test>,
    pub data_sources: HashMap<u16, DataSource>,
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
            data_sources: HashMap::new(),
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
