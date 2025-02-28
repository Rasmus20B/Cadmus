
use std::collections::HashMap;
use std::path::Path;

use crate::emulator3::record_store::*;
use crate::dbobjects::reference::{TableReference, FieldReference, TableOccurrenceReference};
use crate::dbobjects::schema::{Schema, relationgraph::{graph::*, relation::*}};
use crate::dbobjects::file::File;

use crate::hbam2::{page_store::*, api::*};

use super::database::Database;
use super::record_store::Record;
use std::fs::read_to_string;

pub struct DatabaseMgr {
    pub databases: HashMap::<String, Database>,
}

impl DatabaseMgr {
    pub fn new() -> Self {
        Self {
            databases: HashMap::new(),
        }
    }

    pub fn compare_records<'a>(&'a self,
        table1: &Vec<Record>,
        table2: &Vec<Record>,
        relation: &Relation) -> Vec<u32> {
            let mut related_records = vec![];
            for record in table1 {
                for other in table2 {
                    for criteria in &relation.criteria {
                        // TODO: current field value can be cached
                        let cur_field = record.fields.iter().find(|field| field.0 == criteria.field_self).unwrap();
                        let next_field = other.fields.iter().find(|field| field.0 == criteria.field_other).unwrap();
                        if match criteria.comparison {
                            RelationComparison::Equal => cur_field == next_field,
                            RelationComparison::NotEqual => cur_field != next_field,
                            // TODO: Implement the other comparisons
                            _ => todo!("Other comparisons to be implemented"),
                        } {
                            related_records.push(other.id)
                        }
                    }
                }
            }
            related_records
    }

    pub fn get_related_records(&self,
        from: (TableOccurrenceReference, u32),
        to: &TableOccurrenceReference,
        active_database: &str) 
        -> Option<Vec<u32>> {

            let db = self.databases.get(active_database).unwrap();
            let cur_table_ref = &db.file.schema.relation_graph.nodes.iter()
                .find(|source| source.id == from.0.table_occurrence_id)
                .unwrap().base.table_id;

            let cur_records: &Vec<Record> = db.records.records_by_table
                .get(cur_table_ref).unwrap();

            let mut cur_records: Vec<&Record> = vec![&cur_records.iter()
                .find(|record| record.id == from.1).unwrap()];

            println!("Starting records: {:?}", cur_records);
            let node1 = from.0.table_occurrence_id;
            let node2 = to.table_occurrence_id;

            let path = db.file.schema.relation_graph.get_path(node1, node2).unwrap();

            for pair in path.windows(2) {
                let _ = pair[0]; let next = pair[1];

                let next_table_ref = &db.file.schema.relation_graph.nodes.iter()
                    .find(|occ| occ.id == next).unwrap().base;

                let next_db = if next_table_ref.data_source == 0 {
                    &db
                } else {
                    let target_file = &db.file.data_sources
                        .iter()
                        .find(|source| source.id == next_table_ref.data_source)
                        .unwrap().paths[0];

                    let target_file = db.file.working_dir.clone() + "/" + target_file;
                    println!("looking for related records in: {}", target_file);
                    self.databases.get(&target_file).unwrap()
                };

                let next_records = next_db.records.records_by_table.get(&next_table_ref.table_id).unwrap();
                println!("next tables records\n===========");
                // for record in next_records {
                //     println!("{:?}", record);
                // }

                let relation = db.file.schema.relation_graph.nodes
                    .iter()
                    .find(|node| node.id == pair[0]).unwrap().relations
                    .iter()
                    .find(|relation| relation.other_occurrence == pair[1]).unwrap();

                let mut related_records = vec![];
                for record in cur_records {
                    for other in next_records {
                        for criteria in &relation.criteria {
                            // TODO: current field value can be cached
                            let cur_field = record.fields.iter().find(|field| field.0 == criteria.field_self).unwrap();
                            let next_field = other.fields.iter().find(|field| field.0 == criteria.field_other).unwrap();
                            if match criteria.comparison {
                                RelationComparison::Equal => cur_field.1 == next_field.1,
                                RelationComparison::NotEqual => cur_field.1 != next_field.1,
                                _ => todo!("Other comparisons to be implemented"),
                            } {
                                related_records.push(other)
                            }
                        }
                    }
                }
                if related_records.is_empty() {
                    return None
                }
                cur_records = related_records.clone();
            }

            Some(cur_records.iter().map(|record| record.id).collect())
    }

    /* We need to supply information about the record, and record ID itself.
     * This includes: Table occurrence and record ID.
     * We also need to supply information about the field we are looking for:
     * This is just the FieldReference itself. */
    pub fn get_field(&self,
        from: (TableOccurrenceReference, u32),
        to: FieldReference,
        active_database: &str
        ) -> Option<String> {

        let to_occurrence_ref = TableOccurrenceReference {
            data_source: to.data_source,
            table_occurrence_id: to.table_occurrence_id
        };
        let record_ids = match self.get_related_records(from, &to_occurrence_ref, active_database) {
            Some(inner) => inner,
            None => return None,
        };

        let cur_db = self.databases.get(active_database).unwrap();

        let other_db = self.databases.get(
            &cur_db.file.data_sources
                .iter()
                .find(|source| source.id == to.data_source)
                .unwrap().paths[0]
            ).unwrap();

        let other_table = cur_db.file.schema.relation_graph.nodes
            .iter()
            .find(|node| node.id == to.table_occurrence_id)
            .unwrap().base.table_id;

        Some(other_db.records.get_field(other_table, record_ids[0], to.field_id))
    }

    pub fn set_field(&mut self,
        from: (TableOccurrenceReference, u32),
        to: FieldReference,
        active_database: &str,
        value: &str,
        ) -> Result<(), &str> {

        let to_occurrence_ref = TableOccurrenceReference {
            data_source: to.data_source,
            table_occurrence_id: to.table_occurrence_id
        };
        let record_ids = match self.get_related_records(from, &to_occurrence_ref, active_database) {
            Some(inner) => inner,
            None => return Err("No related record exists"),
        };

        let cur_db = self.databases.get(active_database).unwrap();
        let other_db_path = cur_db.file.working_dir.clone() + "/" + &cur_db.file.data_sources.iter().find(|source| source.id == to.data_source).unwrap()
            .paths[0].clone();
        let other_table = cur_db.file.schema.relation_graph.nodes
            .iter()
            .find(|node| node.id == to.table_occurrence_id)
            .unwrap().base.table_id;

        let other_db = self.databases.get_mut(&other_db_path).unwrap();
        other_db.records.set_field(other_table, record_ids[0], to.field_id, value.to_string());
        Ok(())
    }

    pub fn new_record(&mut self,
        table_ref: TableReference,
        active_database: &str) -> u32 {

        let db = self.databases.get_mut(active_database).unwrap();
        db.records.new_record(db.file.schema.tables.iter().find(|table| table.id == table_ref.table_id).unwrap())
    }

    pub fn load_file(&mut self, path: &str) -> &Database {
        if self.databases.contains_key(path) {
            return self.databases.get(path).unwrap()
        }

        if path.ends_with(".cad") {
            // load cad file
            println!("path: {}", path);
            let cadcode = read_to_string(&path).unwrap();
            let mut file = crate::cadlang::compiler::compile_to_file(Path::new(path)).unwrap();
            file.name = path.to_string();
            let database = Database::from_file(file);
            self.databases.insert(path.to_string(), database);
            self.databases.get(path).unwrap()
        } else if path.ends_with(".fmp12") {
            // TODO: load hbam file
            let mut cache = PageStore::new();
            let data_sources_ = get_datasource_catalog(&mut cache, path)
                .into_iter()
                .map(|ds| ds.1)
                .collect::<Vec<_>>();
            let tables_ = get_table_catalog(&mut cache, path)
                .into_iter()
                .map(|table| table.1)
                .collect::<Vec<_>>();
            let relation_graph_ = get_occurrence_catalog(&mut cache, path)
                .into_iter()
                .map(|node| node.1)
                .collect::<Vec<_>>();
            let scripts_ = get_script_catalog(&mut cache, path)
                .into_iter()
                .map(|node| node.1)
                .collect::<Vec<_>>();
            let layouts_ = get_layout_catalog(&mut cache, path)
                .into_iter()
                .map(|node| node.1)
                .collect::<Vec<_>>();

            let parent_dir = Path::new(path).parent().unwrap().to_str().unwrap().to_string();
            let database = Database {
                records: RecordStore::new(&tables_),
                file: File {
                    name: String::new(),
                    working_dir: parent_dir,
                    schema: Schema {
                        tables: tables_,
                        relation_graph: RelationGraph {
                            nodes: relation_graph_,
                        },
                    },
                    data_sources: data_sources_,
                    layouts: layouts_,
                    scripts: scripts_,
                    tests: vec![],
                },
            };
            self.databases.insert(path.to_string(), database);
            self.databases.get(path).unwrap()
        } else {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use std::path::Path;
    use super::super::Emulator;
    use crate::dbobjects::reference::*;
    #[test]
    fn related_records() {
        let mut env = Emulator::new();
        let quotes_path = "test_data/cad_files/multi_file_solution/quotes.cad";
        let customers_path = "test_data/cad_files/multi_file_solution/customers.cad";
        let material_path = "test_data/cad_files/multi_file_solution/materials.cad";
        env.load_file(quotes_path);

        let quotes_to_ref = TableOccurrenceReference {
            data_source: 0,
            table_occurrence_id: 1,
        };
        assert_eq!(env.database_mgr.databases.len(), 3);
        let quote_record = env.database_mgr.new_record(
            TableReference {
                data_source: 0,
                table_id: 1,
            },
            quotes_path
        );

        env.database_mgr.databases.get_mut(quotes_path).unwrap()
            .records.set_field(1, 1, 2, 2.to_string());

        for i in 1..=5 {
            let record_id = env.database_mgr.new_record(
                TableReference {
                    data_source: 0,
                    table_id: 1,
                },
                customers_path
            );

            env.database_mgr.databases.get_mut(customers_path).unwrap()
                .records.set_field(1, record_id, 1, i.to_string())
        }

        let related_records = env.database_mgr.get_related_records((quotes_to_ref.clone(), 1),
            &TableOccurrenceReference {
                data_source: 1,
                table_occurrence_id: 2 
            },
            quotes_path);

        println!("Quotes\n=============");
        for record in env.database_mgr.databases.get(quotes_path).unwrap()
            .records.records_by_table.get(&1).unwrap() {
                println!("{:?}", record);
            }
        println!("Customers\n=============");
        for record in env.database_mgr.databases.get(customers_path).unwrap()
            .records.records_by_table.get(&1).unwrap() {
                println!("{:?}", record);
        }

        assert_eq!(related_records.unwrap().len(), 1);

        let _ = env.database_mgr.set_field(
            (quotes_to_ref.clone(), 1),
            FieldReference {
                data_source: 1,
                table_occurrence_id: 2,
                field_id: 1,
            },
            quotes_path,
            "",
        );

        let related_records = env.database_mgr.get_related_records((quotes_to_ref, 1),
            &TableOccurrenceReference {
                data_source: 1,
                table_occurrence_id: 2 
            },
            quotes_path);

        assert_eq!(related_records, None);
    }
}





