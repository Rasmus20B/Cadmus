use std::collections::HashMap;

use crate::schema::*;

use super::relation_mgr::RelationMgr;

#[derive(Clone, PartialEq, Debug)]
pub struct Field {
    pub name: String,
    pub records: Vec<String>,
}

impl Field {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            records: vec![],
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct TableObject {
    pub name: String,
    pub fields: Vec<Field>,
}

impl TableObject {
    pub fn new(table_name: String) -> Self {
        Self {
            name: table_name,
            fields: vec![]
        }
    }

    pub fn add_blank_record(&mut self) {
        for field in &mut self.fields {
            field.records.push(String::new());
        }
    }

    pub fn get_records_n(&self) -> usize {
        self.fields[0].records.len()
    }

    pub fn delete_record(&mut self, record_id: u16) {
        self.fields.remove(record_id.into());
    }
}

#[derive(Clone, Debug)]
pub struct RelationCriteria {
    join_by: RelationComparison,
    field1: usize,
    field2: usize,
}

#[derive(Clone, Debug)]
pub struct RelatedRecordSet {
    relationship: Vec<RelationCriteria>,
    occurrence: usize,
    records: Vec<usize>,
}

#[derive(Clone)]
pub struct TableOccurrence {
    pub found_set: Vec<usize>,
    pub table_ptr: u16,
    pub record_ptr: usize,
    pub related_records: Vec<RelatedRecordSet>,
}

impl TableOccurrence {
    pub fn new() -> Self {
        Self {
            found_set: vec![],
            table_ptr: 0,
            record_ptr: 0,
            related_records: Vec::<RelatedRecordSet>::new(),
        }
    }
    pub fn get_current_record(&self) -> Result<usize, &str> {
        let res = self.found_set.get(self.record_ptr);
        match res {
            Some(res) => Ok(*res),
            None => Err("No Record found.")
        }
    }
}

/* Database will keep:
 * - Base level records (A list of Records),
 * - Table Occurences which have their own found_set and record handles. */
pub struct Database {
    pub table_occurrences: Vec<TableOccurrence>,
    occurrence_indices: HashMap<String, u16>,
    occurrence_handle: u16,
    pub tables: Vec<TableObject>,
    relation_mgr: RelationMgr,
}

impl Database {
    pub fn new() -> Self {
        Self {
            table_occurrences: vec![],
            occurrence_indices: HashMap::new(),
            occurrence_handle: 0,
            tables: vec![],
            relation_mgr: RelationMgr::new(),
        }
    }

    pub fn clear_records(&mut self) {
        for table in &mut self.tables {
            for field in &mut table.fields {
                field.records.clear();
            }
        }
    }

    pub fn generate_from_fmp12(&mut self, file: &Schema) {

        /* Generate Base Tables */
        let tables_size = file.tables.keys().max().unwrap_or(&0);
        self.tables.resize(*tables_size + 1, TableObject::new("".to_string()));
        for (i, table) in &file.tables {
            let tmp = TableObject {
                name: table.name.clone(),
                fields: vec![],
            };
            self.tables[*i] = tmp;

            let fields_size = table.fields.keys().max().unwrap_or(&0);
            self.tables[*i].fields.resize(*fields_size + 1, Field::new());
            for (j, field) in &table.fields {
                self.tables[*i].fields[*j] = Field {
                    name: field.name.to_string(),
                    records: vec![]
                }
            }
        }

        /* Generate Table Occurrences */
        let occurrence_size = file.table_occurrences.keys().max().unwrap_or(&0);
        self.occurrence_handle = *file.table_occurrences.keys().min().unwrap_or(&0) as u16;
        self.table_occurrences.resize(*occurrence_size + 1, TableOccurrence::new());
        for (i, occurrence) in &file.table_occurrences {
            self.occurrence_indices.insert(
                occurrence.name.clone(),
                *i as u16);

            let tmp = TableOccurrence {
                found_set: vec![],
                record_ptr: 0,
                table_ptr: occurrence.table_actual,
                related_records: Vec::new(),
            };
            self.table_occurrences[*i] = tmp;
        }

        /* Generate Relationships */ 
        for rel in file.relations.values() {
            self.table_occurrences[rel.table1 as usize].related_records.push(
                RelatedRecordSet {
                    occurrence: rel.table2 as usize,
                    relationship: vec![RelationCriteria {
                        field1: rel.criterias[0].field1 as usize,
                        field2: rel.criterias[0].field2 as usize,
                        join_by: rel.criterias[0].comparison.clone(),
                    }],
                    records: vec![],
                }
            );
            self.table_occurrences[rel.table2 as usize].related_records.push(
                RelatedRecordSet {
                    occurrence: rel.table1 as usize,
                    relationship: vec![RelationCriteria {
                        field1: rel.criterias[0].field2 as usize,
                        field2: rel.criterias[0].field1 as usize,
                        join_by: rel.criterias[0].comparison.clone(),
                    }],
                    records: vec![],
                }
            );
            self.relation_mgr.add_relation(rel.table1 as usize, rel.table2 as usize);
        }
    }

    pub fn create_record(&mut self) {
        /* Rules: 
         * - When creating a record, all table_occurences with same table will add it to the found
         * set (even if it doesn't match), and update their record_ptr to look at the new record.
         */ 
        
        let handle = self.occurrence_handle;
        let t = self.table_occurrences[handle as usize].clone();
        let name = self.tables[t.table_ptr as usize].name.clone();
        let table_idx = self.get_current_occurrence().table_ptr;
        let table = self.get_current_table_mut();
        table.add_blank_record();
        let n = table.get_records_n() - 1;
        for occurrence in &mut self.table_occurrences {
            if occurrence.table_ptr == table_idx {
                occurrence.found_set.push(n);
                occurrence.record_ptr = occurrence.found_set.len() - 1;
            }
        }
    }

    pub fn get_found_set_record_field(&self, field: &str) -> &str {
        let occ = self.get_current_occurrence();
        let table = self.get_current_table();

        let cur_idx = occ.found_set[occ.record_ptr];

        for f in &table.fields {
            if f.name == field {
                return &f.records[cur_idx];
            }
        }
        ""
    }

    pub fn get_occurrence(&self, occurrence_handle: usize) -> &TableOccurrence {
        &self.table_occurrences[occurrence_handle]
    }

    pub fn get_occurrence_mut(&mut self, occurrence_handle: usize) -> &mut TableOccurrence {
        &mut self.table_occurrences[occurrence_handle]
    }

    pub fn get_occurrence_by_name(&self, occurrence_handle: &str) -> &TableOccurrence {
        &self.table_occurrences[self.occurrence_indices[occurrence_handle] as usize]
    }

    pub fn get_occurrence_by_name_mut(&mut self, occurrence_handle: &str) -> &mut TableOccurrence {
        &mut self.table_occurrences[self.occurrence_indices[occurrence_handle] as usize]
    }

    pub fn set_current_occurrence(&mut self, occurrence: u16) {
        self.occurrence_handle = occurrence;
    }

    pub fn get_current_occurrence(&self) -> &TableOccurrence {
        &self.table_occurrences[self.occurrence_handle as usize]
    }

    pub fn get_current_occurrence_mut(&mut self) -> &mut TableOccurrence {
        &mut self.table_occurrences[self.occurrence_handle as usize]
    }

    pub fn get_current_table(&self) -> &TableObject {
        &self.tables[self.get_current_occurrence().table_ptr as usize]
    }

    pub fn get_current_table_mut(&mut self) -> &mut TableObject {
        let id = self.get_current_occurrence().table_ptr;
        &mut self.tables[id as usize]
    }

    pub fn get_table(&self, name: &str) -> Option<&TableObject> {
        self.tables.iter().find(|table| table.name == name)
    }

    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut TableObject> {
        self.tables.iter_mut().find(|table| table.name == name)
    }

    pub fn get_records_for_current_table(&self) -> &Vec<Field> {
        &self.tables[self.get_current_occurrence().table_ptr as usize].fields
    }

    pub fn get_field_vals_for_current_table(&self, field: &str) -> &Vec<String> {
        let records = self.tables[self.get_current_occurrence().table_ptr as usize]
            .fields.iter()
            .filter(|x| x.name == field)
            .collect::<Vec<&Field>>();

        &records[0].records
    }

    pub fn get_current_record_field(&self, field: &str) -> &str {
        let occurrence = self.get_current_occurrence();
        let id = occurrence.get_current_record();

        if id.is_err() {
            eprintln!("[-] Record not found.");
            return "";
        }
        let table = occurrence.table_ptr;

        let field = self.tables[table as usize].fields
            .iter()
            .filter(|x| x.name == field)
            .collect::<Vec<&Field>>();

        &field[0].records[id.unwrap()]
    }

    pub fn get_record_by_field(&self, field: &str, record_id: usize) -> Result<&str, &str> {
        let occurrence = self.get_current_occurrence();
        let id = occurrence.get_current_record();
        let table = occurrence.table_ptr;

        if id.is_err() {
            return Err("Record not found.");
        }

        let field = self.tables[table as usize].fields
            .iter()
            .filter(|x| x.name == field)
            .collect::<Vec<&Field>>();

        Ok(&field[0].records[record_id])
    }

    pub fn get_current_record_by_field_mut(&mut self, field: &str) -> Result<&mut str, &str> {
        let occurrence = self.get_current_occurrence().clone();
        let id = occurrence.get_current_record();
        let table = occurrence.table_ptr;

        if id.is_err() {
            return Err("Record not found.");
        }

        let field = self.tables[table as usize].fields
            .iter_mut()
            .enumerate()
            .filter(|x| x.1.name == field)
            .collect::<Vec<_>>()[0].0;

        Ok(&mut self.tables[table as usize].fields[field].records[id.unwrap()])
    }

    pub fn get_current_record_by_table_field(&self, occurrence: &str, field: &str) -> Result<&str, &str> {

        let occurrence = &self.table_occurrences[self.occurrence_indices[occurrence] as usize];
        let id = occurrence.get_current_record();
        let table = occurrence.table_ptr;

        if id.is_err() {
            return Err("Record not found.");
        }

        let field = self.tables[table as usize].fields
            .iter()
            .enumerate()
            .filter(|x| x.1.name == field)
            .collect::<Vec<_>>()[0].0;

        Ok(&self.tables[table as usize].fields[field].records[id.unwrap()])
    }

    pub fn get_current_record_by_table_field_mut(&mut self, occurrence: &str, field: &str) -> Result<&mut String, &str> {

        let occurrence = &self.table_occurrences[self.occurrence_indices[occurrence] as usize];
        let id = occurrence.get_current_record();
        let table = occurrence.table_ptr;

        if id.is_err() {
            return Err("Record not found.");
        }

        let field = self.tables[table as usize].fields
            .iter_mut()
            .enumerate()
            .filter(|x| x.1.name == field)
            .collect::<Vec<_>>()[0].0;

        Ok(&mut self.tables[table as usize].fields[field].records[id.unwrap()])
    }

    pub fn get_related_record_field(&mut self, occurrence: &str, field_target: &str) -> Result<String, &str> {

        let target_idx = self.occurrence_indices[occurrence] as usize;
        let path = self.relation_mgr.get_path(self.occurrence_handle.into(), target_idx);
        if path.is_none() {
            return Err("Cannot access unrelated record.");
        }

        let path_uw = path.clone().unwrap();
        let mut current_set = vec![];

        for (current, next) in path_uw.windows(2).map(|x| (x[0], x[1])) {
            let current_occurrence = self.get_occurrence(current);
            let current_record = current_occurrence.get_current_record();

            if current_record.is_err() {
                return Err("Cannot access unrelated record.");
            }

            let relation = current_occurrence.related_records
                .iter()
                .filter(|x| x.occurrence == next)
                .collect::<Vec<_>>();

            if relation.is_empty() {
                return Err("Cannot access unrelated record.");
            }

            if current_set.is_empty() {
                let tmp = &self.get_current_table()
                    .fields[relation[0].relationship[0].field1]
                    .records[current_record.unwrap()];
                current_set.push((current_record.unwrap(), tmp.to_string()));
            }

            let next_occurrence = self.get_occurrence(next);
            let next_table = &self.tables[next_occurrence.table_ptr as usize];

            let rhs_list = &next_table
                .fields[relation[0].relationship[0].field2]
                .records;

            let mut related_set = vec![];
            for lhs in &mut current_set {
                for rhs in rhs_list.iter().enumerate() {
                    let related = match relation[0].relationship[0].join_by {
                        RelationComparison::Equal => *lhs.1 == *rhs.1,
                        RelationComparison::NotEqual => *lhs.1 != *rhs.1,
                        RelationComparison::Less => *lhs.1 <= *rhs.1.to_string(),
                        RelationComparison::LessEqual => *lhs.1 <= *rhs.1.to_string(),
                        RelationComparison::Greater => *lhs.1 >= *rhs.1.to_string(),
                        RelationComparison::GreaterEqual => *lhs.1 >= *rhs.1.to_string(),
                        RelationComparison::Cartesian => true,
                    };

                    // println!("relation: {}: {} :: {:?} :: {}: {}", current, lhs.1,
                    // relation[0].relationship.join_by, next, rhs.1);
                    if related {
                        related_set.push((rhs.0, rhs.1.to_string()));
                    }
                }
            }
            if related_set.is_empty() {
                return Err("Cannot access unrelated record.");
            }
            current_set.clear();
            current_set.clone_from(&related_set);
        }

        let n = path_uw.last().unwrap();
        let table = self.get_occurrence(*n).table_ptr;

        return Ok(self.tables[table as usize].fields
            .iter()
            .filter(|x| x.name == field_target)
            .collect::<Vec<_>>()[0]
            .records[current_set[0].0].to_string());
    }

    pub fn get_related_record_field_mut(&mut self, occurrence: &str, field: &str) -> &mut str {
        let target_idx = self.occurrence_indices[occurrence] as usize;
        let target_occurrence = &self.table_occurrences[target_idx];
        let current_occurrence = &self.get_current_occurrence();

        let related_record_idx = current_occurrence.related_records
            .iter()
            .filter(|x| x.occurrence == target_idx)
            .take(1)
            .collect::<Vec<_>>()[0].records[0];

        let table_idx = target_occurrence.table_ptr;

        let field = self.tables[table_idx as usize].fields
            .iter()
            .enumerate()
            .filter(|x| x.1.name == field)
            .collect::<Vec<_>>()[0].0;

        &mut self.tables[table_idx as usize].fields[field].records[0]
    }

    pub fn get_nth_related_record_field(&mut self, occurrence: &str, field: &str, n: usize) -> &str {
        let target_idx = self.occurrence_indices[occurrence] as usize;
        let target_occurrence = &self.table_occurrences[target_idx];
        let current_occurrence = &self.get_current_occurrence();

        let related_record_idx = current_occurrence.related_records
            .iter()
            .filter(|x| x.occurrence == target_idx)
            .take(1)
            .collect::<Vec<_>>()[0].records[0];

        let table_idx = target_occurrence.table_ptr;

        let field = self.tables[table_idx as usize].fields
            .iter()
            .enumerate()
            .filter(|x| x.1.name == field)
            .collect::<Vec<_>>()[0].0;

        let records = &self.tables[table_idx as usize].fields[field].records;
        if n >= records.len() {
            &records[records.len()]
        } else {
            &records[n]
        }
    }

    pub fn get_nth_related_record_field_mut(&mut self, occurrence: &str, field: &str, n: usize) -> &mut str {
        let target_idx = self.occurrence_indices[occurrence] as usize;
        let target_occurrence = &self.table_occurrences[target_idx];
        let current_occurrence = &self.get_current_occurrence();

        let related_record_idx = current_occurrence.related_records
            .iter()
            .filter(|x| x.occurrence == target_idx)
            .take(1)
            .collect::<Vec<_>>()[0].records[0];

        let table_idx = target_occurrence.table_ptr;

        let field = self.tables[table_idx as usize].fields
            .iter()
            .enumerate()
            .filter(|x| x.1.name == field)
            .collect::<Vec<_>>()[0].0;

        let records = &mut self.tables[table_idx as usize].fields[field].records;
        let length = records.len();
        if n >= records.len() {
            &mut records[length]
        } else {
            &mut records[n]
        }
    }

    /* called after a "perform_find" type script step */
    pub fn update_found_set(&mut self, records: &[usize]) {
        if records.is_empty() {
            self.reset_found_set();
            return;
        }

        self.reset_found_set();
        let handle = self.get_current_occurrence_mut();
        handle.found_set = records.to_vec();
        handle.record_ptr = 0;
    }

    pub fn reset_found_set(&mut self) {
        self.get_current_occurrence_mut()
            .found_set = self.get_current_table()
                            .fields[0].records.iter()
                            .enumerate()
                            .map(|x| x.0)
                            .collect();
    }

    pub fn goto_record(&mut self, record_id: usize) {
        let set = self.get_current_occurrence_mut();
        if record_id >= set.found_set.len() {
            set.record_ptr = set.found_set.len() - 1;
        } else {
            set.record_ptr = record_id;
        }
    }

    pub fn goto_previous_record(&mut self) {
        let set = self.get_current_occurrence_mut();
        if set.record_ptr > 0 {
            set.record_ptr -= 1;
        }
    }

    pub fn goto_next_record(&mut self) {
        let set = self.get_current_occurrence_mut();
        if set.record_ptr < set.found_set.len() - 1 {
            set.record_ptr += 1;
        }
    }

    pub fn goto_first_record(&mut self) {
        let set = self.get_current_occurrence_mut();
        set.record_ptr = 0;
    }

    pub fn goto_last_record(&mut self) {
        let set = self.get_current_occurrence_mut();
        set.record_ptr = set.found_set.len() - 1;
    }
}

