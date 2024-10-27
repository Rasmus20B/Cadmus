use crate::schema::{Schema, RelationCriteria};
use super::parser::BindingList;

#[derive(Debug)]
pub enum ValidationErr {
    UnknownTable(String),
    UnknownTableOccurrence(String),
    UnknownField(String),
}

pub fn validate_table_occurrence_references(schema: &mut Schema, bindings: &BindingList) -> Result<(), ValidationErr> {
    for ref mut table_occurrence in &mut schema.table_occurrences {
        table_occurrence.1.table_actual = match bindings.tables
            .iter()
            .find(|table| table.1 == table_occurrence.1.name) {
                Some(inner) => inner.0 as u16,
                None => {
                    return Err(ValidationErr::UnknownTable(table_occurrence.1.name.clone()))
                }
        }
    }

    Ok(())
}

pub fn validate_layout_references(schema: &mut Schema, bindings: &BindingList) -> Result<(), ValidationErr> {
    for ref mut layout in &mut schema.layouts {
        layout.1.table_occurrence = match bindings.table_occurrences
            .iter()
            .find(|occ| occ.1 == layout.1.name) {
                Some(inner) => inner.0,
                None => {
                    return Err(ValidationErr::UnknownTableOccurrence(layout.1.table_occurrence_name.clone()));
                }
        }
    }
    Ok(())
}

pub fn validate_relation_references(schema: &mut Schema, bindings: &BindingList) -> Result<(), ValidationErr> {
    for ref mut relation in &mut schema.relations {
        let table_occ1 = match bindings.table_occurrences
            .iter()
            .find(|occ| occ.1 == relation.1.table1_name) {
                Some(inner) => inner.0 as u16,
                None => {
                    return Err(ValidationErr::UnknownTableOccurrence(relation.1.table1_name.clone()))
                }
            };
        let table_occ2 = match schema.table_occurrences
            .iter()
            .find(|occ| occ.1.name == relation.1.table2_name) {
                Some(inner) => *inner.0 as u16,
                None => {
                    return Err(ValidationErr::UnknownTableOccurrence(relation.1.table2_name.clone()))
                }
            };

        let table1 = schema.table_occurrences.get(&(table_occ1 as usize)).unwrap().table_actual;
        let table2 = schema.table_occurrences.get(&(table_occ2 as usize)).unwrap().table_actual;
        relation.1.table1 = table1;
        relation.1.table2 = table2;


        for criteria in &mut relation.1.criterias {
            let (field1_, field2_, comp) = match criteria {
                RelationCriteria::ByName { field1, field2, comparison } => {
                    (field1, field2, comparison)
                },
                RelationCriteria::ById { .. } => {
                    continue
                },
            };

            let field1_ = field1_.split("::").collect::<Vec<_>>()[1];
            let field2_ = field2_.split("::").collect::<Vec<_>>()[1];

            let mut field1_idx = 0;
            let mut field2_idx = 0;
            for field in &schema.tables.get(&(table1 as usize)).unwrap().fields {
                if field.1.name == *field1_ {
                    field1_idx = *field.0;
                }
            }

            for field in &schema.tables.get(&(table2 as usize)).unwrap().fields {
                if field.1.name == *field2_ {
                    field2_idx = *field.0;
                }
            }
            *criteria = RelationCriteria::ById { field1: field1_idx as u16, field2: field2_idx as u16, comparison: *comp };

        }
        
    }

    Ok(())
}


pub fn validate(schema: &mut Schema, bindings: &BindingList) -> Result<(), ValidationErr> {
    validate_table_occurrence_references(schema, bindings)?;
    validate_layout_references(schema, bindings)?;
    validate_relation_references(schema, bindings)?;
    Ok(())
}
