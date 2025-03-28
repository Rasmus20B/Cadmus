
use std::path::Component;

use crate::dbobjects::{
    file::*,
    layout::Layout, 
    scripting::{
        script::*,
        instructions::*,
        arguments::*
    },
    reference::{
        FieldReference,
        ScriptReference,
        TableReference,
        TableOccurrenceReference,
    },
    calculation::Calculation,
    schema::{
        field::*, relationgraph::{
            graph::RelationGraph,
            relation::{
                Relation, RelationComparison, RelationCriteria
            },
            table_occurrence::TableOccurrence
        }, table::Table, Schema
    },
};
use super::{cadscript::proto_instruction::ProtoScriptSelection, staging::*};
use super::cadscript::proto_instruction::{ProtoInstruction, ProtoFieldSelection};

use std::collections::{HashSet, HashMap, BTreeMap};

use super::{lexer::*, parser::*};

use std::path::Path;
use std::fs::read_to_string;

pub fn compile_external_cad_data_sources(stage: &Stage, working_dir: &Path) -> HashMap<u32, Stage> {
    let mut result = HashMap::new();

    let mut set = HashSet::<String>::new();

    for (i, source) in &stage.data_sources {
        for path in &source.paths {
            let full_path = working_dir.join(Path::new(path));
            println!("Looking at path: {:?}", full_path.as_path());
            if !set.contains(&full_path.to_str().unwrap().to_string()) {
                set.insert(full_path.to_str().unwrap().to_string());
                let stage = parse(&lex(&read_to_string(&Path::new(&full_path)).unwrap()).unwrap()).unwrap();
                result.insert(*(i) as u32, stage);
            }
        }
    }
    result
}

pub fn generate_table_occurrence_refs(stage: &Stage, externs: &HashMap<u32, Stage>) -> Vec<TableOccurrence> {
    
    let mut result = Vec::<TableOccurrence>::new();
    for (i, occurrence) in &stage.table_occurrences {
        println!("looking @ {}", occurrence.base_table.value);
        if occurrence.data_source.is_none() {
            // look in the current file's schema for the correct object
            for table in &stage.tables {
                println!("TABLE: {}", table.1.name.value);
            }
            let table = stage.tables.iter()
                .find(|table| table.1.name.value == occurrence.base_table.value)
                .map(|table| table.1)
                .unwrap();

            let tmp = TableOccurrence {
                id: (*i) as u32,
                name: occurrence.name.value.clone(),
                base: TableReference {
                    data_source: 0 as u32,
                    table_id: table.id as u32,
                },
                relations: vec![],
            };
            result.push(tmp);
        } else {
            let extern_id = stage.data_sources.iter()
                .find(|occ| occ.1.name == occurrence.data_source.as_ref().unwrap().value)
                .map(|occ| occ.1)
                .unwrap();

            for (_, table) in &externs.get(&(extern_id.id)).unwrap().tables {
                if table.name.value == occurrence.base_table.value {
                    let tmp = TableOccurrence {
                        id: (*i) as u32,
                        name: occurrence.name.value.clone(),
                        base: TableReference {
                            data_source: extern_id.id as u32,
                            table_id: table.id as u32,
                        },
                        relations: vec![],
                    };
                    result.push(tmp);
                }
            }
        }
    }
    result
}

pub fn generate_relation_refs(stage: &Stage, live_occurrences: &mut Vec<TableOccurrence>, externs: &HashMap<u32, Stage>) {
    for (i, relation) in &stage.relations {
        let mut tmp1 = Relation {
            id: (*i) as u32,
            other_occurrence: 0,
            criteria: vec![],
        };
        let mut tmp2 = Relation {
            id: (*i) as u32,
            other_occurrence: 0,
            criteria: vec![],
        };

        let mut occ1_id = 0;
        let mut occ2_id = 0;

        for criteria in &relation.criterias {
            let occurrence1 = live_occurrences.iter()
                .find(|occ| occ.name == criteria.occurrence1.value)
                .unwrap();
            let occurrence2 = live_occurrences.iter()
                .find(|occ| occ.name == criteria.occurrence2.value)
                .unwrap();

            occ1_id = occurrence1.id;
            occ2_id = occurrence2.id;

            tmp1.other_occurrence = occ2_id;
            tmp2.other_occurrence = occ1_id;

            let table1 = if occurrence1.base.data_source == 0 {
                stage.tables.iter()
                    .find(|table| table.1.id as u32 == occurrence1.base.table_id)
                    .unwrap()
            } else {
                let source = externs.get(&(occurrence1.base.data_source)).unwrap();
                source.tables.iter()
                    .find(|table| table.1.id as u32 == occurrence1.base.table_id)
                    .unwrap()
            };

            let table2 = if occurrence2.base.data_source == 0 {
                stage.tables.iter()
                    .find(|table| table.1.id as u32 == occurrence2.base.table_id)
                    .unwrap()
            } else {
                let source = externs.get(&(occurrence2.base.data_source)).unwrap();
                source.tables.iter()
                    .find(|table| table.1.id as u32 == occurrence2.base.table_id)
                    .unwrap()
            };

            let field1 = table1.1.fields
                .iter()
                .find(|field| field.1.name.value == criteria.field1.value)
                .map(|field| field.0)
                .unwrap();
            let field2 = table2.1.fields
                .iter()
                .find(|field| field.1.name.value == criteria.field2.value)
                .map(|field| field.0)
                .unwrap();

            let comp = match criteria.comparison {
                RelationComparison::Equal => RelationComparison::Equal,
                RelationComparison::NotEqual => RelationComparison::NotEqual,
                RelationComparison::Less => RelationComparison::Less,
                RelationComparison::LessEqual => RelationComparison::LessEqual,
                RelationComparison::Greater => RelationComparison::Greater,
                RelationComparison::GreaterEqual => RelationComparison::GreaterEqual,
                RelationComparison::Cartesian => RelationComparison::Cartesian,
            };

            let crit1 = RelationCriteria {
                field_self: (*field1) as u32,
                field_other: (*field2) as u32,
                comparison: comp,
            };
            let crit2 = RelationCriteria {
                field_self: (*field2) as u32,
                field_other: (*field1) as u32,
                comparison: comp.mirrored(),
            };

            tmp1.criteria.push(crit1);
            tmp2.criteria.push(crit2);
        }
        live_occurrences.iter_mut().find(|occ| occ.id == occ1_id).unwrap().relations.push(tmp1);
        live_occurrences.iter_mut().find(|occ| occ.id == occ2_id).unwrap().relations.push(tmp2);
    }
}

pub fn generate_table_occurrences(stage: &Stage, externs: &HashMap<u32, Stage> ) -> Vec<TableOccurrence> {
    let mut live_occurrences = generate_table_occurrence_refs(stage, externs);
    generate_relation_refs(stage, &mut live_occurrences, &externs);

    for occ in &live_occurrences {
        println!("{}. {}", occ.id, occ.name);
        for rel in &occ.relations {
            println!("{:?}", rel);
        }
    }
    live_occurrences
}

pub fn load_externs(stage: &Stage, working_dir: &Path) -> HashMap::<u32, Stage> {
    compile_external_cad_data_sources(stage, working_dir)
}

pub fn encode_calculation(code: &str, schema: &Stage, externs: &HashMap<u32, Stage>, graph: &RelationGraph) -> Calculation {
    let tokens = crate::dbobjects::calculation::lex_text(code);
    let resolved = tokens
        .into_iter()
        .map(|tok| match tok {
            crate::dbobjects::calculation::token::Token::FieldReference(occurrence, field) => { 
                let node = graph.nodes
                    .iter()
                    .find(|search_occ| search_occ.name == occurrence)
                    .unwrap();

                println!("NODE FOUND IN THE REFERENCE::{:?}", node);

                // If the table is local to the file
                let table = if node.base.data_source == 0 {
                    schema.tables
                        .get(&(node.base.table_id as u16))
                        .unwrap()
                } else {
                    externs.get(&node.base.data_source).unwrap()
                        .tables.get(&(node.base.table_id as u16))
                        .unwrap()
                };

                let field = table.fields.iter()
                    .find(|search_field| search_field.1.name.value == field)
                    .unwrap();

                println!("Resolved field ref to: {:?}", FieldReference {
                    data_source: node.base.data_source,
                    table_occurrence_id: node.id,
                    field_id: field.1.id as u32,
                });
                return crate::dbobjects::calculation::token::Token::ResolvedFieldReference(
                    FieldReference {
                        data_source: node.base.data_source,
                        table_occurrence_id: node.id,
                        field_id: field.1.id as u32,
                    }
                );
            },
            _ => tok,
            })
        .collect::<Vec<crate::dbobjects::calculation::token::Token>>();

    Calculation::from_tokens(&resolved)

}

fn generate_tables(stage: &Stage, externs: &HashMap<u32, Stage>, graph: &RelationGraph) -> Vec<Table> {
    let mut tables = vec![];

    for (i, table) in &stage.tables {
        let mut tmp = Table {
            id: table.id as u32,
            name: table.name.value.clone(),
            created_by: String::from("admin"),
            modified_by: String::from("admin"),
            comment: String::new(),
            fields: BTreeMap::new(),
        };

        for(j, field) in &table.fields {
            let nomodify_ = field.autoentry.nomodify;
            let autoentry_ = match &field.autoentry.definition {
                StagedAutoEntryType::NA => AutoEntryType::NA,
                StagedAutoEntryType::Data(data) => AutoEntryType::Data(data.clone()),
                StagedAutoEntryType::LastVisited => AutoEntryType::LastVisited,
                StagedAutoEntryType::Serial { 
                    next,
                    increment,
                    trigger 
                } => AutoEntryType::Serial { 
                    next: *next,
                    increment: *increment,
                    trigger: trigger.clone(),
                },
                StagedAutoEntryType::Lookup { from, to } => {
                    AutoEntryType::NA
                }
                StagedAutoEntryType::Creation(preset) => AutoEntryType::Creation(preset.clone()),
                StagedAutoEntryType::Modification(preset) => AutoEntryType::Modification(preset.clone()),
                StagedAutoEntryType::Calculation { code, noreplace } => {
                    AutoEntryType::Calculation { 
                        code: encode_calculation(code, stage, externs, graph),
                        noreplace: *noreplace 
                    }
                }
            };

            tmp.fields.insert((*j) as u32, Field {
                id: (*j) as u32,
                name: field.name.value.clone(),
                repetitions: field.repetitions as u8,
                dtype: field.dtype.clone(),
                autoentry: AutoEntry {
                    definition: autoentry_, 
                    nomodify: nomodify_,
                },
                validation: Validation {
                    checks: vec![],
                    message: field.validation.message.clone(),
                    trigger: field.validation.trigger.clone(),
                    user_override: field.validation.user_override,
                },
                created_by: String::new(),
                modified_by: String::new(),
                global: field.global,
            });
        }
        tables.push(tmp)
    }

    tables
}

pub fn build_schema(stage: &Stage, externs: &HashMap<u32, Stage>) -> Schema {
    let mut result = Schema::new();

    result.relation_graph.nodes = generate_table_occurrences(stage, &externs);
    result.tables = generate_tables(stage, &externs, &result.relation_graph);
    result
}

fn build_script_objects(stage: &Stage, externs: &HashMap<u32, Stage>, graph: &RelationGraph) -> Vec<(u32, Script)> {
    let mut finished_scripts = vec![];
    for (t, (i, script)) in stage.scripts.iter().map(|script| (0, script)).chain(stage.tests.iter().map(|test| (1, test)).into_iter()) {
        let mut tmp = Script {
            id: (*i) as u32,
            name: script.name.clone(),
            args: vec![],
            instructions: vec![],
            metadata: crate::dbobjects::metadata::Metadata {
                created_by: String::new(),
                modified_by: String::new(),
            }
        };

        tmp.instructions = script.instructions.iter().enumerate().map(|(i, instr)| {
           ScriptStep { id: i as u32, instruction: match instr {
                ProtoInstruction::PerformScript { script, args } => {
                    println!("DOes get here into the PERFORM SCRIPT SCIRPTIPSIPCISPCIPSCSCPI");
                    let script_ = match script {
                        ProtoScriptSelection::UnresolvedReference { data_source, script } => {
                            let ds = stage.data_sources.iter()
                                .find(|ds| ds.1.name == *data_source)
                                .unwrap();

                            let script_id_ = externs.iter()
                                .find(|e| *e.0 == ds.1.id)
                                .unwrap()
                                .1.scripts.iter()
                                .find(|script_| script_.1.name == *script)
                                .unwrap()
                                .0;

                            ScriptSelection::FromList( ScriptReference {
                                data_source: ds.1.id,
                                script_id: (*script_id_) as u32,
                            })
                        }
                        _ => { todo!() }
                    };
                    Instruction::PerformScript { 
                        script: script_,
                        args: encode_calculation(&args.0, stage, externs, graph) 
                    }
                },
                ProtoInstruction::NewRecordRequest => Instruction::NewRecordRequest,
                ProtoInstruction::SetField { field, value, repetition } => Instruction::SetField {
                    field: match field {
                        ProtoFieldSelection::UnresolvedReference { occurrence, field } => {
                            let node = graph.nodes
                                .iter()
                                .find(|search_occ| search_occ.name == *occurrence)
                                .unwrap();

                            let occ_id = node.id;
                            let (external_ds, table_id) = (node.base.data_source, node.base.table_id);

                            let field_id_ = if external_ds == 0 {
                                stage.tables.get(&(table_id as u16)).unwrap()
                                    .fields.iter()
                                    .find(|search| search.1.name.value == *field)
                                    .map(|f| f.1.id)
                                    .unwrap()
                            } else {
                                let data_source = externs.get(&external_ds).unwrap();
                                data_source.tables.get(&(table_id as u16)).unwrap()
                                    .fields.iter()
                                    .find(|search| search.1.name.value == *field)
                                    .map(|f| f.1.id)
                                    .unwrap()
                            };

                            FieldReference { 
                                data_source: external_ds,
                                table_occurrence_id: occ_id, 
                                field_id: field_id_ as u32,
                            }

                        },
                    },
                    value: encode_calculation(value.0.as_str(), stage, externs, graph),
                    repetition: encode_calculation(repetition.0.as_str(), stage, externs, graph),
                },
                ProtoInstruction::SetVariable { name, value, repetition }  => Instruction::SetVariable { 
                    name: name.to_string(),
                    value: encode_calculation(value.0.as_str(), stage, externs, graph),
                    repetition: encode_calculation(repetition.0.as_str(), stage, externs, graph)
                },
                ProtoInstruction::Loop => Instruction::Loop,
                ProtoInstruction::EndLoop => Instruction::EndLoop,
                ProtoInstruction::ExitLoopIf { condition } => Instruction::ExitLoopIf {
                    condition: encode_calculation(condition.0.as_str(), stage, externs, graph),
                },
                _ => Instruction::Print,
           }
           }
        }).collect();
        finished_scripts.push((t, tmp));
    }
    finished_scripts
}

pub fn build_file(stage: &Stage, working_dir: &Path) -> File {
    let mut layouts_ = vec![];
    let externs = load_externs(stage, working_dir);
    let schema_ = build_schema(&stage, &externs);

    for (i, layout) in &stage.layouts {
        let occurrence_id = schema_.relation_graph.nodes
            .iter()
            .find(|l| l.name == layout.base_occurrence.value).unwrap().id;

        let tmp = Layout {
            id: (*i) as u32,
            name: layout.name.value.clone(),
            occurrence: TableOccurrenceReference {
                data_source: 0,
                table_occurrence_id: occurrence_id,
            }
        };
        layouts_.push(tmp);
    }

    let (scripts_, tests_): (Vec<_>, Vec<_>) = build_script_objects(stage, &externs, &schema_.relation_graph)
        .into_iter().partition(|(t, _)| *t == 0);

    File {
        name: String::new(),
        working_dir: working_dir.to_str().unwrap().to_string(),
        schema: schema_,
        data_sources: stage.data_sources.iter().map(|ds| ds.1.clone()).collect(),
        layouts: layouts_,
        scripts: scripts_.into_iter().map(|(_, script)| script).collect(),
        tests: tests_.into_iter().map(|(_, script)| script).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dbobjects::{schema::relationgraph::graph::*, data_source::*};
    use std::fs::read_to_string;
    #[test]
    fn basic_multi_file() {
        let file = read_to_string("./test_data/cad_files/multi_file_solution/quotes.cad").unwrap();
        let mut stage = parse(&lex(&file).unwrap()).unwrap();
        let mut working_dir = Path::new("./test_data/cad_files/multi_file_solution/quotes.cad").parent().unwrap();
        let file = build_file(&mut stage, working_dir);

        let expected_data_sources = vec![
                DataSource {
                    id: 1,
                    name: String::from("Customers"),
                    dstype: DataSourceType::Cadmus,
                    paths: vec![
                        String::from("customers.cad")
                    ],
                },
                DataSource {
                    id: 2,
                    name: String::from("Materials"),
                    dstype: DataSourceType::Cadmus,
                    paths: vec![
                        String::from("materials.cad")
                    ],
                },
            ];
        assert_eq!(file.data_sources, expected_data_sources);

        let expected_layouts = vec![
            Layout {
                id: 1,
                name: String::from("Quotes"),
                occurrence: TableOccurrenceReference {
                    data_source: 0,
                    table_occurrence_id: 1,
                }
            },
            Layout {
                id: 2,
                name: String::from("MatJoin"),
                occurrence: TableOccurrenceReference {
                    data_source: 0,
                    table_occurrence_id: 5,
                }
            }
        ];
        assert_eq!(file.layouts, expected_layouts);

        let expected_tables = vec![
            Table {
                id: 1,
                name: String::from("Quotes"),
                comment: String::new(),
                created_by: String::from("admin"),
                modified_by: String::from("admin"),
                fields: BTreeMap::from([
                    (1, Field {
                        id: 1,
                        name: String::from("id"),
                        dtype: DataType::Number,
                        created_by: String::new(),
                        modified_by: String::new(),
                        validation: Validation {
                            checks: vec![],
                            message: String::new(),
                            trigger: ValidationTrigger::OnEntry,
                            user_override: true,
                        },
                        autoentry: AutoEntry {
                            definition: AutoEntryType::Serial { 
                                next: 1,
                                increment: 1,
                                trigger: SerialTrigger::OnCreation 
                            },
                            nomodify: false,
                        },
                        repetitions: 1,
                        global: false,
                    }),
                    (2, Field {
                        id: 2,
                        name: String::from("customer_id"),
                        dtype: DataType::Text,
                        created_by: String::new(),
                        modified_by: String::new(),
                        validation: Validation {
                            checks: vec![],
                            message: String::new(),
                            trigger: ValidationTrigger::OnEntry,
                            user_override: true,
                        },
                        autoentry: AutoEntry {
                            definition: AutoEntryType::NA,
                            nomodify: false,
                        },
                        repetitions: 1,
                        global: false,
                    }),
                    (3, Field {
                        id: 3,
                        name: String::from("price"),
                        dtype: DataType::Number,
                        created_by: String::new(),
                        modified_by: String::new(),
                        validation: Validation {
                            checks: vec![],
                            message: String::new(),
                            trigger: ValidationTrigger::OnEntry,
                            user_override: true,
                        },
                        autoentry: AutoEntry {
                            definition: AutoEntryType::NA,
                            nomodify: false,
                        },
                        repetitions: 1,
                        global: false,
                    }),
                ]),
            },
            Table {
                id: 2,
                name: String::from("MaterialJoin"),
                comment: String::new(),
                created_by: String::from("admin"),
                modified_by: String::from("admin"),
                fields: BTreeMap::from([
                    (1, Field {
                        id: 1,
                        name: String::from("quote_id"),
                        dtype: DataType::Number,
                        created_by: String::new(),
                        modified_by: String::new(),
                        validation: Validation {
                            checks: vec![],
                            message: String::new(),
                            trigger: ValidationTrigger::OnEntry,
                            user_override: true,
                        },
                        autoentry: AutoEntry {
                            definition: AutoEntryType::NA,
                            nomodify: false,
                        },
                        repetitions: 1,
                        global: false,
                    }),
                    (2, Field {
                        id: 2,
                        name: String::from("material_id"),
                        dtype: DataType::Number,
                        created_by: String::new(),
                        modified_by: String::new(),
                        validation: Validation {
                            checks: vec![],
                            message: String::new(),
                            trigger: ValidationTrigger::OnEntry,
                            user_override: true,
                        },
                        autoentry: AutoEntry {
                            definition: AutoEntryType::NA,
                            nomodify: false,
                        },
                        repetitions: 1,
                        global: false,
                    }),
                ]),
            },
        ];

        assert_eq!(file.schema.tables.len(), expected_tables.len());
        for (actual, expected) in file.schema.tables.iter().zip(expected_tables) {
            assert_eq!(actual.fields.len(), expected.fields.len());
            for i in actual.fields.keys() {
                assert_eq!(actual.fields[&i], expected.fields[&i]);
            }
        }

        let expected_graph = RelationGraph {
            nodes: vec![
                TableOccurrence {
                    id: 1,
                    name: String::from("Quotes"),
                    base: TableReference {
                        data_source: 0,
                        table_id: 1,
                    },
                    relations: vec![
                        Relation {
                            id: 1,
                            other_occurrence: 2,
                            criteria: vec![
                                RelationCriteria {
                                    field_self: 2,
                                    field_other: 1,
                                    comparison: RelationComparison::Equal,
                                }
                            ],
                        },
                        Relation {
                            id: 2,
                            other_occurrence: 5,
                            criteria: vec![
                                RelationCriteria {
                                    field_self: 1,
                                    field_other: 1,
                                    comparison: RelationComparison::Equal,
                                }
                            ],
                        }
                    ],
                },
                TableOccurrence {
                    id: 2,
                    name: String::from("Customers"),
                    base: TableReference {
                        data_source: 1,
                        table_id: 1,
                    },
                    relations: vec![
                        Relation {
                            id: 1,
                            other_occurrence: 1,
                            criteria: vec![
                                RelationCriteria {
                                    field_self: 1,
                                    field_other: 2,
                                    comparison: RelationComparison::Equal,
                                }
                            ],
                        }
                    ],
                },
                TableOccurrence {
                    id: 3,
                    name: String::from("backup"),
                    base: TableReference {
                        data_source: 1,
                        table_id: 2,
                    },
                    relations: vec![],
                },
                TableOccurrence {
                    id: 4,
                    name: String::from("Materials"),
                    base: TableReference {
                        data_source: 2,
                        table_id: 1,
                    },
                    relations: vec![
                        Relation {
                            id: 3,
                            other_occurrence: 5,
                            criteria: vec![
                                RelationCriteria {
                                    field_self: 1,
                                    field_other: 2,
                                    comparison: RelationComparison::Equal,
                                }
                            ],
                        },
                    ],
                },
                TableOccurrence {
                    id: 5,
                    name: String::from("MaterialJoin"),
                    base: TableReference {
                        data_source: 0,
                        table_id: 2,
                    },
                    relations: vec![
                        Relation {
                            id: 2,
                            other_occurrence: 1,
                            criteria: vec![
                                RelationCriteria {
                                    field_self: 1,
                                    field_other: 1,
                                    comparison: RelationComparison::Equal,
                                }
                            ],
                        },
                        Relation {
                            id: 3,
                            other_occurrence: 4,
                            criteria: vec![
                                RelationCriteria {
                                    field_self: 2,
                                    field_other: 1,
                                    comparison: RelationComparison::Equal,
                                }
                            ],
                        }
                    ],
                },
            ],
        };
        assert_eq!(file.schema.relation_graph, expected_graph);
    }
}



