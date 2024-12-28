use std::collections::hash_map::HashMap;
use super::{token::Token, error::CompileErr, parser::FMObjType};
use crate::schema::{DBObjectReference, RelationCriteria, Relation, RelationComparison, Schema, Table, Field, LayoutFM, AutoEntry, AutoEntryType, Validation, ValidationType, ValidationTrigger, TableOccurrence, DataType, AutoEntryDataPresets, SerialTrigger, Test, Script};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StagedTable {
    pub id: u16,
    pub name: Token,
    pub fields: HashMap<u16, StagedField>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StagedValidationType {
    NotEmpty,
    Unique,
    Required,
    MemberOf(Token),
    Range{start: usize, end: usize},
    Calculation(String),
    MaxChars(usize),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StagedValidation {
    pub trigger: ValidationTrigger,
    pub user_override: bool,
    pub checks: Vec<StagedValidationType>,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StagedAutoEntryType {
    NA,
    Serial { next: usize, increment: usize, trigger: SerialTrigger },
    Lookup { from: Token, to: Token },
    Creation(AutoEntryDataPresets),
    Modification(AutoEntryDataPresets),
    LastVisited,
    Data(String),
    Calculation{code: String, noreplace: bool},
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StagedAutoEntry {
    pub nomodify: bool,
    pub definition: StagedAutoEntryType
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StagedField {
    pub id: u16,
    pub name: Token,
    pub dtype: DataType,
    pub validation: StagedValidation,
    pub autoentry: StagedAutoEntry,
    pub global: bool,
    pub repetitions: usize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StagedRelationCriteria {
    pub field1: Token,
    pub field2: Token,
    pub comparison: RelationComparison,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StagedRelation {
    pub id: u16,
    pub table1: String,
    pub table2: String,
    pub criterias: Vec<StagedRelationCriteria>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StagedOccurrence {
    pub id: u16,
    pub name: Token,
    pub base_table: Token,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StagedValueListSortBy {
    FirstField,
    SecondField,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StagedValueListDefinition {
    CustomValues(Vec<String>),
    FromField { field1: Token, 
        field2: Option<Token>, 
        from: Option<Token>, 
        sort: StagedValueListSortBy, 
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StagedValueList {
    pub id: u16,
    pub name: Token,
    pub definition: StagedValueListDefinition,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StagedLayout {
    pub id: u16,
    pub name: Token,
    pub base_occurrence: Token,
}

#[derive(Debug, Clone)]
pub struct Stage {
    pub tables: HashMap<u16, StagedTable>,
    pub table_occurrences: HashMap<u16, StagedOccurrence>,
    pub relations: HashMap<u16, StagedRelation>,
    pub value_lists: HashMap<u16, StagedValueList>,
    pub layouts: HashMap<u16, StagedLayout>,
    pub scripts: HashMap<u16, Script>,
    pub tests: HashMap<u16, Test>,
}

impl Stage {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            table_occurrences: HashMap::new(),
            relations: HashMap::new(),
            value_lists: HashMap::new(),
            layouts: HashMap::new(),
            scripts: HashMap::new(),
            tests: HashMap::new(),
        }
    }

    pub fn to_schema(&self) -> Result<Schema, Vec<CompileErr>> {
        let mut result = Schema::new();
        let mut errs = Vec::<CompileErr>::new();

        for (id_, table) in &self.tables {
            let name_ = table.name.value.clone();

            let mut fields_ = HashMap::new();
            for field in &table.fields {
                let mut checks_ = vec![];
                for check in &field.1.validation.checks {
                    let tmp = match check {
                        StagedValidationType::NotEmpty => ValidationType::NotEmpty,
                        StagedValidationType::Unique => ValidationType::Unique,
                        StagedValidationType::Required => ValidationType::Required,
                        StagedValidationType::Range { start, end } => ValidationType::Range { start: *start, end: *end },
                        StagedValidationType::Calculation(calc) => ValidationType::Calculation(calc.clone()),
                        StagedValidationType::MaxChars(max) => ValidationType::MaxChars(*max),
                        StagedValidationType::MemberOf(value_list) => {
                            ValidationType::MemberOf(value_list.value.clone())
                        },
                    };

                    checks_.push(tmp);
                }
                let validation_ = Validation { 
                    message: field.1.validation.message.clone(),
                    trigger: field.1.validation.trigger.clone(),
                    user_override: field.1.validation.user_override,
                    checks: checks_,
                };

                let autoentrytype = match &field.1.autoentry.definition {
                    StagedAutoEntryType::NA => AutoEntryType::NA,
                    StagedAutoEntryType::Serial { next, increment, trigger }  => AutoEntryType::Serial {
                        next: *next, increment: *increment, trigger: trigger.clone()
                    },
                    StagedAutoEntryType::Data(data) => AutoEntryType::Data(data.to_string()),
                    StagedAutoEntryType::LastVisited => AutoEntryType::LastVisited,
                    StagedAutoEntryType::Creation(preset) => AutoEntryType::Creation(preset.clone()),
                    StagedAutoEntryType::Modification(preset) => AutoEntryType::Modification(preset.clone()),
                    StagedAutoEntryType::Calculation { code, noreplace } => AutoEntryType::Calculation { code: code.clone(), noreplace: *noreplace },
                    StagedAutoEntryType::Lookup { from, to } => {

                        let from_occ = match self.table_occurrences.iter().find(|occ| occ.1.name.value == from.value) {
                            Some(inner) => *inner.0,
                            None => {
                                errs.push(CompileErr::UndefinedReference { 
                                    construct: FMObjType::TableOccurrence,
                                    token: from.clone(),
                                });
                                continue
                            },
                        };

                        let to_parts = to.value.split("::").collect::<Vec<_>>();
                        let to_occ = to_parts[0];
                        let to_field = to_parts[1];

                        let (occ_id, occ_base_table) = match self.table_occurrences.iter()
                            .find(|occ| occ.1.name.value == to_occ) {
                                Some(inner) => (inner.0, inner.1.base_table.value.clone()),
                                None => {
                                    errs.push(CompileErr::UndefinedReference {
                                        construct: FMObjType::TableOccurrence,
                                        token: to.clone(),
                                    });

                                    continue;
                                }
                            };

                        let table_actual = match self.tables.iter()
                            .find(|table| table.1.name.value == occ_base_table) {
                                Some(inner) => &inner.1.fields,
                                None => {
                                    errs.push(CompileErr::UndefinedReference {
                                        construct: FMObjType::TableOccurrence,
                                        token: to.clone(),
                                    });
                                    continue;
                                }
                            };



                        AutoEntryType::Lookup { 
                            from: DBObjectReference {
                                data_source: 0,
                                top_id: from_occ,
                                inner_id: 0,
                            }, 
                            to: DBObjectReference {
                                data_source: 0,
                                top_id: from_occ,
                                inner_id: 0,
                            },                      
                        }
                    }
                };

                let autoentry_ = AutoEntry {
                    nomodify: field.1.autoentry.nomodify,
                    definition: autoentrytype,
                };

                let tmp = Field {
                    id: (*field.0) as usize,
                    name: field.1.name.value.clone(),
                    dtype: field.1.dtype.clone(),
                    global: field.1.global,
                    validation: validation_,
                    autoentry: autoentry_,
                    repetitions: field.1.repetitions,
                    created_by: String::from("admin"),
                    modified_by: String::from("admin"),
                };

                fields_.insert(tmp.id, tmp);
            }
            
            result.tables.insert((*id_) as usize, Table {
                id: *(id_) as usize,
                name: name_,
                fields: fields_,
                created_by: String::from("admin"),
                modified_by: String::from("admin"),
            });
        }

        for (id_, occurrence) in &self.table_occurrences {
            let name_ = occurrence.name.value.clone();
            let base_table_ = occurrence.base_table.clone();
            match self.tables.iter().find(|table| table.1.name.value == base_table_.value) {
                Some(inner) => {
                    result.table_occurrences.insert((*id_) as usize, TableOccurrence {
                        id: (*id_) as usize,
                        name: name_,
                        base_table: DBObjectReference {
                            data_source: 0,
                            top_id: inner.1.id,
                            inner_id: 0,
                        },
                        created_by: String::from("admin"),
                        modified_by: String::from("admin"),
                    });
                }
                None => {
                    errs.push(CompileErr::UndefinedReference { construct: FMObjType::Table, token: base_table_ });
                }
            }
        }

        for (_, relation) in &self.relations {
            let mut tmp = Relation {
                id: 0,
                criterias: vec![],
                table1: DBObjectReference { data_source: 0, top_id: 0, inner_id: 0 },
                table2: DBObjectReference { data_source: 0, top_id: 0, inner_id: 0 },
            };

            for crit in &relation.criterias {
                let mut field1: usize = 0;
                let comparison = RelationComparison::Equal;
                let parts = crit.field1.value.split("::").collect::<Vec<_>>();
                let occurrence = parts[0];
                let field1_name = parts[1];
                let mut table1 = String::new();
                let mut occ1_id = 0;

                match self.table_occurrences.iter().find(|occ| occ.1.name.value == occurrence) {
                    Some(inner) => {
                        table1 = inner.1.base_table.value.clone();
                        occ1_id = (*inner.0) as usize;
                    },
                    None => {
                        errs.push(CompileErr::UndefinedReference { construct: FMObjType::TableOccurrence, token: crit.field1.clone() });
                    }
                }

                let table_handle = match self.tables.iter().find(|table| table.1.name.value == table1) {
                    Some(inner) => {
                        inner
                    },
                    None => {
                        continue;
                    }
                };

                match table_handle.1.fields.iter().find(|field| field.1.name.value == field1_name) {
                    Some(inner) => {
                        field1 = (*inner.0) as usize;
                    },
                    None => {
                        errs.push(CompileErr::UndefinedReference { construct: FMObjType::Field, token: crit.field1.clone() });
                    }
                }

                tmp.table1 = DBObjectReference {
                    data_source: 0,
                    top_id: (*table_handle.0),
                    inner_id: 0,
                };


                let mut field2: usize = 0;
                let parts = crit.field2.value.split("::").collect::<Vec<_>>();
                let occurrence = parts[0];
                let field2_name = parts[1];
                let mut table2 = String::new();
                let mut occ2_id = 0;

                match self.table_occurrences.iter().find(|occ| occ.1.name.value == occurrence) {
                    Some(inner) => {
                        table2 = inner.1.base_table.value.clone();
                        occ2_id = (*inner.0) as usize;
                    },
                    None => {
                        errs.push(CompileErr::UndefinedReference { construct: FMObjType::TableOccurrence, token: crit.field1.clone() });
                    }
                }

                let table_handle = match self.tables.iter().find(|table| table.1.name.value == table2) {
                    Some(inner) => {
                        inner
                    },
                    None => {
                        continue;
                    }
                };

                match table_handle.1.fields.iter().find(|field| field.1.name.value == field2_name) {
                    Some(inner) => {
                        field2 = (*inner.0) as usize;
                    },
                    None => {
                        errs.push(CompileErr::UndefinedReference { construct: FMObjType::Field, token: crit.field1.clone() });
                    }
                }


                tmp.table2 = DBObjectReference {
                    data_source: 0,
                    top_id: (*table_handle.0),
                    inner_id: 0,
                };

                tmp.criterias.push(RelationCriteria::ById {
                    field1: field1 as u16,
                    field2: field2 as u16,
                    comparison: crit.comparison
                });
            }
            result.relations.insert(relation.id as usize, tmp);
        }

        for layout in &self.layouts {
            let (id, occ_name) = match self.table_occurrences.iter().find(|occ| occ.1.name.value == layout.1.base_occurrence.value) {
                Some(inner) => (
                    inner.1.id,
                    inner.1.base_table.value.clone()
                ),
                None => {
                    errs.push(CompileErr::UndefinedReference {
                        construct: FMObjType::TableOccurrence,
                        token: layout.1.base_occurrence.clone(),
                    });
                    continue;
                },
            };

            result.layouts.insert((*layout.0) as usize, LayoutFM {
                id: (*layout.0) as usize,
                name: layout.1.name.value.clone(),
                table_occurrence_name: occ_name,
                table_occurrence: DBObjectReference {
                    data_source: 0,
                    top_id: id,
                    inner_id: 0,
                },
            });
        }

        result.tests.extend(&mut self.tests.clone().into_iter());

        result.scripts.extend(&mut self.scripts.clone().into_iter());

        if errs.is_empty() {
            Ok(result)
        } else {
            Err(errs)
        }

    }
}

#[cfg(test)]
mod tests {
    use crate::cadlang::{parser::parse, lexer::lex};
    use std::path::Path;
    #[test]
    fn multi_crit_relation_to_real() {
        let code = std::fs::read_to_string(Path::new("test_data/cad_files/multi_criteria_relation.cad"))
            .expect("Unable to read test cad file.");
        let tokens = lex(&code).expect("Tokenisation failed.");
        let schema = parse(&tokens).expect("Parsing failed.").to_schema();
    }
}
