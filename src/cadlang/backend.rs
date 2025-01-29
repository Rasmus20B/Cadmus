


use crate::dbobjects::{reference::TableReference, schema::{Schema, relationgraph::{relation::{Relation, RelationCriteria, RelationComparison}, table_occurrence::TableOccurrence}}};
use super::staging::Stage;

use std::collections::{HashSet, HashMap};

use super::{lexer::*, parser::*};

use std::path::Path;
use std::fs::read_to_string;

pub fn compile_external_cad_data_sources(schema: &Stage) -> HashMap<usize, Stage> {
    let mut result = HashMap::new();

    let mut set = HashSet::<String>::new();

    for (i, source) in &schema.data_sources {
        if !set.contains(&source.filename) {
            set.insert(source.filename.clone());
            let stage = parse(&lex(&read_to_string(&Path::new(&source.filename)).unwrap()).unwrap()).unwrap();
            result.insert(*(i) as usize, stage);
        }
    }
    result
}

pub fn generate_table_occurrence_refs(schema: &Stage, externs: &HashMap<usize, Stage>) -> Vec<TableOccurrence> {
    
    let mut result = Vec::<TableOccurrence>::new();
    for (i, occurrence) in &schema.table_occurrences {
        println!("{}", occurrence.base_table.value);
        if occurrence.data_source.is_none() {
            // look in the current file's schema for the correct object
            let table = schema.tables.iter()
                .find(|table| table.1.name.value == occurrence.name.value)
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
            let extern_id = schema.data_sources.iter()
                .find(|occ| occ.1.name == occurrence.data_source.as_ref().unwrap().value)
                .map(|occ| occ.1)
                .unwrap();

            println!("extern: {} == {}", occurrence.data_source.as_ref().unwrap().value, extern_id.id);
            println!("Tables in {}: ", extern_id.name);
            for (_, table) in &externs.get(&extern_id.id).unwrap().tables {
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

    for occ in &result {
        println!("external reference for {}:: {:?}", occ.name, occ.base);
    }
    result
}

pub fn generate_relation_refs(schema: &Stage, live_occurrences: &mut Vec<TableOccurrence>, externs: &HashMap<usize, Stage>) {
    for (i, relation) in &schema.relations {
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
                schema.tables.iter()
                    .find(|table| table.1.id as u32 == occurrence1.base.table_id)
                    .unwrap()
            } else {
                let source = externs.get(&(occurrence1.base.data_source as usize)).unwrap();
                source.tables.iter()
                    .find(|table| table.1.id as u32 == occurrence1.base.table_id)
                    .unwrap()
            };

            let table2 = if occurrence2.base.data_source == 0 {
                schema.tables.iter()
                    .find(|table| table.1.id as u32 == occurrence2.base.table_id)
                    .unwrap()
            } else {
                let source = externs.get(&(occurrence2.base.data_source as usize)).unwrap();
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

            println!("{}::{}({}, {}, {}) <- {:?} -> {}::{}({}, {}, {})", 
                criteria.occurrence1.value,
                criteria.field1.value,
                occurrence1.base.data_source,
                table1.1.id,
                field1,
                criteria.comparison,
                criteria.occurrence2.value,
                criteria.field2.value,
                occurrence2.base.data_source,
                table2.1.id,
                field2,
                );

            let comp = match criteria.comparison {
                crate::schema::RelationComparison::Equal => RelationComparison::Equal,
                crate::schema::RelationComparison::NotEqual => RelationComparison::NotEqual,
                crate::schema::RelationComparison::Less => RelationComparison::Less,
                crate::schema::RelationComparison::LessEqual => RelationComparison::LessEqual,
                crate::schema::RelationComparison::Greater => RelationComparison::Greater,
                crate::schema::RelationComparison::GreaterEqual => RelationComparison::GreaterEqual,
                crate::schema::RelationComparison::Cartesian => RelationComparison::Cartesian,
            };

            let crit1 = RelationCriteria {
                field_self: (*field1) as u32,
                field_other: (*field2) as u32,
                comparison: comp,
            };
            let crit2 = RelationCriteria {
                field_self: (*field2) as u32,
                field_other: (*field1) as u32,
                comparison: comp,
            };

            tmp1.criteria.push(crit1);
            tmp2.criteria.push(crit2);
        }
        live_occurrences.iter_mut().find(|occ| occ.id == occ1_id).unwrap().relations.push(tmp1);
        live_occurrences.iter_mut().find(|occ| occ.id == occ2_id).unwrap().relations.push(tmp2);
    }
}

pub fn generate_external_refs(schema: &Stage) -> (HashMap::<usize,Stage>, Vec<TableOccurrence>) {
    let externs = compile_external_cad_data_sources(schema);
    let mut live_occurrences = generate_table_occurrence_refs(schema, &externs);
    generate_relation_refs(schema, &mut live_occurrences, &externs);

    //for occ in &live_occurrences {
    //    println!("{}. {}", occ.id, occ.name);
    //    for rel in &occ.relations {
    //        println!("{:?}", rel);
    //    }
    //}

    (externs, live_occurrences)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;
    #[test]
    fn basic_multi_file() {
        let file = read_to_string("./test_data/cad_files/multi_file_solution/quotes.cad").unwrap();
        let mut stage = parse(&lex(&file).unwrap()).unwrap();
        generate_external_refs(&mut stage);
    }
}




