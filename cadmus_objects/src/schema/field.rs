use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::{calculation::Calculation, file::File, reference::*};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum SerialTrigger {
    OnCreation,
    OnCommit,
}

impl core::fmt::Display for SerialTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OnCreation => write!(f, "on_creation"),
            Self::OnCommit => write!(f, "on_commit"),
        }
    }
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

impl core::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Text => write!(f, "Text"),
            DataType::Number => write!(f, "Number"),
            DataType::Time => write!(f, "Time"),
            DataType::Date => write!(f, "Date"),
            DataType::Timestamp => write!(f, "Text"),
            DataType::Container => write!(f, "Container"),
        }
    }
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
    Calculation{code: Calculation, noreplace: bool},
}

impl AutoEntryType {
    pub fn to_cad(&self) -> String {
        match self {
            Self::Serial { next, increment, trigger } => String::from(
                format!("        serial = {{\n            generate = {},\n            next = {},\n            increment = {},\n        }}\n", trigger, next, increment)),
            _ => String::new()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AutoEntry {
    pub nomodify: bool,
    pub definition: AutoEntryType
}

impl AutoEntry {
    pub fn to_cad(&self) -> String {
        let mut buffer = String::new();
        buffer.push_str(&format!("        nomodify = {}\n", self.nomodify));
        buffer.push_str(&self.definition.to_cad());
        buffer
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ValidationTrigger {
    OnEntry,
    OnCommit,
}

impl core::fmt::Display for ValidationTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OnEntry => write!(f, "on_entry"),
            Self::OnCommit => write!(f, "on_commit"),
        }
    }
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

impl ValidationType {
    pub fn to_cad(&self) -> String {
        match self {
            Self::NotEmpty => "        not_empty = true,\n".to_string(),
            Self::Unique => "        unique = true,\n".to_string(),
            Self::Required => "        required = true,\n".to_string(),
            _ => todo!()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Validation {
    pub trigger: ValidationTrigger,
    pub user_override: bool,
    pub checks: Vec<ValidationType>,
    pub message: String,
}

impl Validation {
    pub fn to_cad(&self) -> String {
        let mut buffer = String::new();
        buffer.push_str(&format!("        trigger = {},\n", self.trigger));
        buffer.push_str(&format!("        user_override = {},\n", self.user_override));
        buffer.push_str(&format!("        message = \"{}\",\n", self.message));
        for check in &self.checks {
            buffer.push_str(&check.to_cad());
        }

        buffer
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Field {
    pub id: u32,
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
    pub fn new(id_: u32, name_: String) -> Self {
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

    pub fn to_cad(&self, file: &File, externs: &HashMap<usize, File>) -> String {
        let mut buffer = String::new();
        buffer.push_str(&format!("field %{} {} = {{\n", self.id, self.name));

        buffer.push_str(&format!("        datatype = {},\n", self.dtype));
        buffer.push_str(&self.autoentry.to_cad());
        buffer.push_str(&self.validation.to_cad());

        buffer.push_str("    }");
        buffer
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

