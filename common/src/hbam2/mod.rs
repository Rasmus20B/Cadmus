mod kv_cache;
pub(crate) mod bplustree;
mod keyvalue;
mod page;
pub mod page_store;
pub(crate) mod chunk;
pub mod path;
mod view;

use crate::{dbobjects::{
    calculation::Calculation, data_source::*, layout::*, metadata::Metadata, reference::*, schema::{relationgraph::relation::Relation, Schema}, scripting::{
        instructions::*, script::*
    }
    },
    hbam2::bplustree::get_view_from_key,
    util::encoding_util::{
        fm_string_decrypt,
        get_path_int
    }
};

use bplustree::{search_key, BPlusTreeErr};
use page_store::PageStore;
use path::HBAMPath;

use crate::dbobjects::{file::File, schema::{relationgraph::{graph::RelationGraph, table_occurrence::TableOccurrence, relation::*}, table::Table, field::Field}};

use std::collections::HashMap;
use std::collections::BTreeMap;

use std::path::Path;

pub type Key = Vec<String>;
pub type Value = String;

#[derive(Debug, PartialEq, Eq)]
pub struct KeyValue {
    pub key: Vec<Vec<u8>>,
    pub value: Vec<u8>,
}

pub fn emit_file(file: &str) {
    let mut cache = PageStore::new();
    bplustree::print_tree(&mut cache, file);
}

pub struct Context {
    cache: PageStore,
}

impl Context {
    pub fn new() -> Context {
        Self {
            cache: PageStore::new()
        }
    }
    pub fn get_schema_contents(&mut self, file: &str) -> File {

        File {
            name: file.to_string(),
            data_sources: get_datasource_catalog(&mut self.cache, file).into_iter().map(|(_, datasource)| datasource).collect(),
            layouts: get_layout_catalog(&mut self.cache, file).into_iter().map(|(_, layout)| layout).collect(),
            schema: Schema {
                tables: get_table_catalog(&mut self.cache, file).into_iter().map(|(_, table)| table).collect(),
                relation_graph: RelationGraph {
                    nodes: get_occurrence_catalog(&mut self.cache, file).into_iter().map(|(_, occurrence)| occurrence).collect(),
                },
            },
            scripts: get_script_catalog(&mut self.cache, file).into_iter().map(|(_, script)| script).collect(),
            tests: vec![],
            working_dir: Path::new(file).parent().unwrap().to_str().unwrap().to_string(),
        }
    }
}

pub fn get_schema_contents(cache: &mut PageStore, file: &str) -> File {

    File {
        name: file.to_string(),
        data_sources: get_datasource_catalog(cache, file).into_iter().map(|(_, datasource)| datasource).collect(),
        layouts: get_layout_catalog(cache, file).into_iter().map(|(_, layout)| layout).collect(),
        schema: Schema {
            tables: get_table_catalog(cache, file).into_iter().map(|(_, table)| table).collect(),
            relation_graph: RelationGraph {
                nodes: get_occurrence_catalog(cache, file).into_iter().map(|(_, occurrence)| occurrence).collect(),
            },
        },
        scripts: get_script_catalog(cache, file).into_iter().map(|(_, script)| script).collect(),
        tests: vec![],
        working_dir: Path::new(file).parent().unwrap().to_str().unwrap().to_string(),
    }
}

pub fn get_table_catalog(cache: &mut PageStore, file: &str) -> HashMap<usize, Table> {
    let view_option = match get_view_from_key(&HBAMPath::new(vec![&[3], &[16], &[5]]), cache, file)
        .expect("Unable to get table info from file.") {
            Some(inner) => inner,
            None => {
                return HashMap::new()
            }
        };

    let mut result = HashMap::new();
    for dir in view_option.get_dirs().unwrap() {
        let path_id =  &dir.path.components[dir.path.components.len()-1];
        let id_ = get_path_int(&dir.path.components[dir.path.components.len()-1]);
        let field_path = HBAMPath::new(vec![path_id.as_slice(), &[3], &[5]]);

        let field_view = match get_view_from_key(&field_path, cache, file)
            .expect("Unable to get fields for table.") {
                Some(inner) => inner,
                None => {
                    continue
                }
        };
        let mut fields_ = BTreeMap::new();

        for field in field_view.get_dirs().unwrap() {
            let path_id = field.path.components.last().unwrap();
            let id_ = get_path_int(path_id);

            let field = Field::new(id_ as u32, fm_string_decrypt(field.get_value(16).expect("Unable to get field name.")))
                .created_by(fm_string_decrypt(field.get_value(64513).expect("Unable to get created by for field.")))
                .modified_by(fm_string_decrypt(field.get_value(64514).expect("Unable to get modified by for field.")));

            fields_.insert(id_ as u32, field);
        }

        result.insert(id_, Table {
            id: id_ as u32,
            name: fm_string_decrypt(dir.get_value(16).unwrap()),
            created_by: fm_string_decrypt(dir.get_value(64513).unwrap()),
            modified_by: fm_string_decrypt(dir.get_value(64513).unwrap()),
            comment: String::new(),
            fields: fields_,
        });
    }
    result
}

pub fn get_occurrence_catalog(cache: &mut PageStore, file: &str) -> HashMap<usize, TableOccurrence> {
    let view_option = match get_view_from_key(&HBAMPath::new(vec![&[3], &[17], &[5]]), cache, file)
        .expect("Unable to get table info from file.") {
            Some(inner) => inner,
            None => return HashMap::new()
        };
    let mut occurrences = HashMap::<usize, TableOccurrence>::new();

    // 1. Get all occurrences as they are in HashMap.
    // 2. Work through the relationship directory, using the definition for the relationship
    //    defined at reference::2;
    //
    //
    for dir in view_option.get_dirs().unwrap() {
        let path_id = dir.path.components.last().unwrap();
        let id_ = get_path_int(path_id) - 128;
        let definition = dir.get_value(2).unwrap();

        let tmp = TableOccurrence {
            id: id_ as u32,
            name: fm_string_decrypt(dir.get_value(16).unwrap()),
            base: TableReference {
                data_source: 0,
                table_id: definition[6] as u32,
            },
            relations: vec![],
        };

        occurrences.insert(id_, tmp);
    }

    // We have the table occurrences with their IDs stored at this point.

    let relation_storage = match get_view_from_key(&HBAMPath::new(vec![&[3], &[251], &[5]]), cache, file).unwrap() {
        Some(inner) => inner,
        None => return occurrences,
    };

    for relation_dir in relation_storage.get_dirs().unwrap() {

        let top_definition = relation_dir.get_value(2).unwrap();
        let from = top_definition[4];
        let to = top_definition[9];


        println!("Definition of relation: {:?}", top_definition);
        let criteria_dir = relation_dir.get_dir_relative(&mut HBAMPath::new(vec![&[3]])).unwrap();
        let criterias = match criteria_dir.get_all_simple_keyvalues() {
            Some(inner) => inner,
            None => { break }
        };

        let id_ = relation_dir.path.components.last().unwrap();

        let mut from_relation = Relation {
            id: get_path_int(id_) as u32,
            other_occurrence: to as u32,
            criteria: vec![],
        };

        let mut to_relation = Relation {
            id: get_path_int(id_) as u32,
            other_occurrence: from as u32,
            criteria: vec![],
        };

        for criteria in criterias {
            let comparison_ = match criteria.1[0] {
                0 => RelationComparison::Equal,
                1 => RelationComparison::NotEqual,
                2 => RelationComparison::Greater,
                3 => RelationComparison::GreaterEqual,
                4 => RelationComparison::Less,
                5 => RelationComparison::LessEqual,
                6 => RelationComparison::Cartesian,
                _ => unreachable!()
            };

            let start1 = 2_usize;
            let len1 = criteria.1[1] as usize;
            let start2 = start1 + len1 + 1_usize;
            let len2 = criteria.1[start1 + len1] as usize;
            let n1 = get_path_int(&criteria.1[start1..start1 + len1]);
            let n2 = get_path_int(&criteria.1[start2..start2 + len2]);
            let from_field = n1 as u16 - 128;
            let to_field = n2 as u16 - 128;

            from_relation.criteria.push(
                RelationCriteria {
                    field_self: from_field as u32,
                    field_other: to_field as u32,
                    comparison: comparison_,
                }
            );

            to_relation.criteria.push(
                RelationCriteria {
                    field_self: to_field as u32,
                    field_other: from_field as u32,
                    comparison: comparison_.mirrored(),
                }
            );
        }

        let from_occurrence = &mut occurrences.get_mut(&(from as usize)).unwrap().relations;
        from_occurrence.push(from_relation);
        let to_occurrence = &mut occurrences.get_mut(&(to as usize)).unwrap().relations;
        to_occurrence.push(to_relation);
    }
    occurrences
}

pub fn get_datasource_catalog(cache: &mut PageStore, file: &str) -> HashMap::<usize, DataSource> {
    let mut result = HashMap::new();
    let datasource_view = match get_view_from_key(&HBAMPath::new(vec![&[32], &[5]]), cache, file).expect("Unable to read data source view.") {
        Some(inner) => inner,
        None => { return result }
    };

    let id_list_view = match datasource_view.get_dirs() {
        Some(inner) => inner,
        None => { return result }
    };

    for source in id_list_view {
        let name = source.get_value(16).unwrap();
        let definition = source.get_value(130).unwrap();
        let id_ = get_path_int(source.path.components.last().unwrap());

        let source_type = &definition[0..=3];
        let typename_size = definition[4] as usize;
        let start = 5;
        let end = start + typename_size;
        let typename = &definition[start..=end];

        let filename_size = definition[end + 1] as usize;
        let start = end + 2;
        let end = start + filename_size;
        let filename = &definition[start..end];
        result.insert(id_, DataSource {
            id: id_ as u32,
            name: fm_string_decrypt(name),
            paths: vec![fm_string_decrypt(filename)],
            dstype: DataSourceType::FileMaker
        });
    }
    result
}

pub fn get_script_catalog(cache: &mut PageStore, file: &str) -> HashMap::<usize, Script> {

    let mut result = HashMap::new();

    let script_catalog_view = match get_view_from_key(&HBAMPath::new(vec![&[17], &[5]]), cache, file).expect("Unable to read script catalog") {
        Some(inner) => inner,
        None => { return result }
    };

    let script_views = match script_catalog_view.get_dirs() {
        Some(inner) => inner,
        None => { return result }
    };

    for chunk in &script_catalog_view.chunks {
        println!("{:?}", chunk);
    }

    for script_view in script_views {
        let id_ = get_path_int(script_view.path.components.last().unwrap());
        let name_ = fm_string_decrypt(script_view.get_value(16).unwrap());
        println!("{}", name_);
        let code = script_view.get_value(4).unwrap();
        println!("===============================");
        //for chunk in &script_view.chunks {
        //    println!("{}::{:?}", script_view.path, chunk);
        //}

        let mut steps = Vec::<ScriptStep>::new();

        for step in code.chunks(28) {
            let mut args = Vec::<String>::new();
            let step_id = step[2];
            let instruction = step[21] as usize;
            let step_storage = match script_view.get_dir_relative(&mut HBAMPath::new(vec![&[5], &[step_id]])) {
                Some(inner) => inner,
                None => continue
            };


            match instruction {
                /* SetVariable */ 141 => {
                    let variable = step_storage
                        .get_dir_relative(&mut HBAMPath::new(vec![&[0, 0]])).unwrap();
                    let variable_name = fm_string_decrypt(variable.get_value(1).unwrap());

                    let expr_storage = step_storage
                        .get_dir_relative(&mut HBAMPath::new(vec![&[0, 1], &[5]])).unwrap();
                    let expr = expr_storage.get_value(5).unwrap();

                    steps.push(
                        ScriptStep {
                            id: step_id as u32,
                            instruction: Instruction::SetVariable {
                                name: variable_name,
                                value: Calculation(expr.to_vec()),
                                repetition: Calculation(vec![]),
                            },
                    });
                },
                _ => continue
            }
        }

        let created_by_ = script_view.get_value(64513).unwrap();
        let modified_by_ = script_view.get_value(64514).unwrap();

        let script = Script {
            name: name_,
            id: id_ as u32,
            instructions: steps,
            metadata: Metadata {
                created_by: fm_string_decrypt(created_by_), 
                modified_by: fm_string_decrypt(modified_by_),
            },
            args: vec![],
        };
        result.insert(id_, script);
    }

    result
}

pub fn get_layout_catalog(cache: &mut PageStore, file: &str) -> HashMap::<usize, Layout> {
    let mut result = HashMap::new();
    let layout_view = match get_view_from_key(&HBAMPath::new(vec![&[4], &[1], &[7]]), cache, file).expect("Unable to read layout catalog") {
        Some(inner) => inner,
        None => { return result }
    };

    for layout in layout_view.get_dirs().unwrap() {
        let id_ = get_path_int(layout.path.components.last().unwrap());
        let meta_definition = layout.get_value(2).unwrap();
        let name_ = fm_string_decrypt(layout.get_value(16).unwrap());

        result.insert(id_, Layout {
            id: id_ as u32,
            name: name_,
            occurrence: TableOccurrenceReference {
                data_source: 0,
                table_occurrence_id: meta_definition[1] as u32,
            },
        });
    }
    result
}

pub fn get_keyvalue<'a>(key: &HBAMPath, store: &'a mut PageStore, file: &'a str) -> Result<Option<KeyValue>, BPlusTreeErr> {
    // Get KeyVal pair from HBAM in byte form.
    search_key(key, store, file)
}


pub fn get_data<'a>(key: Key) -> Result<KeyValue, BPlusTreeErr> {
    unimplemented!()
}

pub fn set_keyvalue<'a>(key: Key, val: Value) -> Result<(), BPlusTreeErr> {
    // setting a keyvalue to a Page
    // Process involes
    // 1. get a copy of the page from the block store
    // 2. set the key and check if there are chunks after it.
    // 3. If there are instructions after it, then split the page
    // 3.5 push page with new key, and new page with keys after the new one. 
    // 4. else just push the page as is to the cache.
    unimplemented!()
}


#[cfg(test)]
mod tests {

    use super::{get_keyvalue, get_table_catalog, get_script_catalog, get_datasource_catalog, KeyValue, PageStore};
    use crate::hbam2::{get_occurrence_catalog, path::HBAMPath};
    use crate::dbobjects::schema::relationgraph::{table_occurrence::TableOccurrence, relation::*};
    use crate::dbobjects::reference::TableReference;
    #[test]
    fn get_keyval_test() {

        let key = HBAMPath::new(vec![
            &[3], &[17], &[1], &[0]
        ]);
        let expected = KeyValue {
            key: key.components.clone(),
            value: vec![3, 208, 0, 1],
        };

        let mut cache = PageStore::new();

        let result = get_keyvalue(&key, &mut cache, "test_data/fmp_files/blank.fmp12").expect("Unable to get keyval");

        assert_eq!(result, Some(expected));
    }

    #[test]
    fn get_table_catalog_test() {
        let mut cache = PageStore::new();
        let result = get_table_catalog(&mut cache, "test_data/fmp_files/blank.fmp12");
        assert_eq!(1, result.len());
        for (_, table) in result {
            println!("Table: {:?}", table);
            for (_, field) in table.fields {
               println!("field: {}", field.name);
            }
        }
    }
    #[test]
    fn get_table_catalog_split_page_test() {
        let mut cache = PageStore::new();
        let result = get_table_catalog(&mut cache, "test_data/fmp_files/mixed.fmp12");
        assert_eq!(2, result.len());

        assert_eq!(result.get(&129).unwrap().fields.len(), 5);
        for (_, table) in result {
            println!("Table: {:?}", table.name);
            for (_, field) in table.fields {
               println!("   field: {}", field.name);
            }
        }
    }

    #[test]
    fn get_data_source_catalog_test() {
        let mut cache = PageStore::new();
        let result = get_datasource_catalog(&mut cache, "test_data/fmp_files/mixed.fmp12");

        assert_eq!(2, result.len());

        for (_, ds) in result {
            println!("{} :: {:?}", ds.name, ds.paths);
        }
    }

    #[test]
    fn get_script_catalog_test() {
        let mut cache = PageStore::new();
        let result = get_script_catalog(&mut cache, "test_data/fmp_files/mixed.fmp12");

        assert_eq!(3, result.len());
    }

    #[test]
    fn get_table_occurrence_catalog_test() {
       let mut cache = PageStore::new();
       let occurrences = get_occurrence_catalog(&mut cache, "test_data/fmp_files/relation.fmp12");
       assert_eq!(4, occurrences.len());
       assert_eq!(*occurrences.get(&1).unwrap(), TableOccurrence {
           id: 1,
           name: String::from("blank"),
           base: TableReference{
               data_source: 0, 
               table_id: 1, 
           },
           relations: vec![
               Relation {
                   id: 1,
                   other_occurrence: 2,
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
                   other_occurrence: 3,
                   criteria: vec![
                       RelationCriteria {
                           field_self: 6,
                           field_other: 1,
                           comparison: RelationComparison::Equal,
                       }
                   ],

               },

           ],
       });
       assert_eq!(*occurrences.get(&2).unwrap(), TableOccurrence {
           id: 2,
           name: String::from("blank 2"),
           base: TableReference {
               data_source: 0,
               table_id: 1,
           },
           relations: vec![
               Relation {
                   id: 1,
                   other_occurrence: 1,
                   criteria: vec![
                       RelationCriteria {
                           field_self: 1,
                           field_other: 1,
                           comparison: RelationComparison::Equal,
                       }
                   ],
               },
           ],
       });
    }
}
