use serde::{Serialize, Deserialize};

use crate::dbobjects::{reference::*, calculation::Calculation};

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
    Lookup { from: TableReference, to: FieldReference },
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
    Calculation(Calculation),
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
    pub repetitions: u8,
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

    pub fn repetitions(mut self, repetitions_: u8) -> Self {
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
