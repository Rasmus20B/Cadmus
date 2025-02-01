use crate::{hbam2::bplustree::get_view_from_key, schema::{AutoEntry, AutoEntryType, DBObjectReference, DataSource, DataSourceType, DataType, Field, LayoutFM, Relation, RelationComparison, RelationCriteria, Script, Table, TableOccurrence, Validation, ValidationTrigger}, util::{calc_bytecode::*, encoding_util::{fm_string_decrypt, get_path_int}}, fm_script_engine::fm_script_engine_instructions::*};

use super::{bplustree::{self, search_key, BPlusTreeErr}, page_store::PageStore, path::HBAMPath};

use std::collections::hash_map::HashMap;

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
        let mut fields_ = HashMap::new();

        for field in field_view.get_dirs().unwrap() {
            let path_id = field.path.components.last().unwrap();
            let id_ = get_path_int(path_id);

            let field = Field::new(id_, fm_string_decrypt(field.get_value(16).expect("Unable to get field name.")))
                .created_by(fm_string_decrypt(field.get_value(64513).expect("Unable to get created by for field.")))
                .modified_by(fm_string_decrypt(field.get_value(64514).expect("Unable to get modified by for field.")));

            fields_.insert(id_, field);
        }

        result.insert(id_, Table {
            id: id_,
            name: fm_string_decrypt(dir.get_value(16).unwrap()),
            created_by: fm_string_decrypt(dir.get_value(64513).unwrap()),
            modified_by: fm_string_decrypt(dir.get_value(64513).unwrap()),
            fields: fields_,
        });
    }
    result
}

pub fn get_occurrence_catalog(cache: &mut PageStore, file: &str) -> (HashMap<usize, TableOccurrence>, HashMap<usize, Relation>) {
    let view_option = match get_view_from_key(&HBAMPath::new(vec![&[3], &[17], &[5]]), cache, file)
        .expect("Unable to get table info from file.") {
            Some(inner) => inner,
            None => return (HashMap::new(), HashMap::new())
        };
    let mut relations = HashMap::<usize, Relation>::new();
    let mut occurrences = HashMap::<usize, TableOccurrence>::new();

    for dir in view_option.get_dirs().unwrap() {
        let path_id = dir.path.components.last().unwrap();
        let id_ = get_path_int(path_id);
        let definition = dir.get_value(2).unwrap();
        occurrences.insert(id_, TableOccurrence {
            id: id_,
            name: fm_string_decrypt(dir.get_value(16).unwrap()),
            created_by: fm_string_decrypt(dir.get_value(64513).unwrap()),
            modified_by: fm_string_decrypt(dir.get_value(64514).unwrap()),
            base_table: DBObjectReference {
                data_source: 0,
                top_id: definition[6] as u16 + 128,
                inner_id: 0,
            }
        });

        let storage = match dir.get_dir_relative(&mut HBAMPath::new(vec![&[251]])) {
            Some(inner) => inner,
            None => { continue; }
        };

        let relation_definitions = storage.get_simple_data().unwrap();
        for def in relation_definitions {
            let table1 = DBObjectReference { 
                data_source: 0,
                top_id: id_ as u16,
                inner_id: 0,
            };
            let table2 = DBObjectReference { 
                data_source: 0,
                top_id: def[2] as u16 + 128,
                inner_id: 0,
            };               

            let tmp = Relation::new(def[4] as usize, table1, table2);
            relations.insert(tmp.id, tmp);
        }
    }

    let relation_option = match get_view_from_key(&HBAMPath::new(vec![&[3], &[251], &[5]]), cache, file).unwrap() {
        Some(inner) => inner,
        None => return (occurrences, relations)
    };

    for relation_dir in relation_option.get_dirs().unwrap() {

        let criteria_dir = relation_dir.get_dir_relative(&mut HBAMPath::new(vec![&[3]])).unwrap();
        let criterias = match criteria_dir.get_all_simple_keyvalues() {
            Some(inner) => inner,
            None => { break }
        };

        let id_ = relation_dir.path.components.last().unwrap();


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
            let field1_ = n1 as u16 - 128;
            let field2_ = n2 as u16 - 128;
            relations.get_mut(&get_path_int(id_)).unwrap()
                .criterias.push(RelationCriteria::ById { field1: field1_, field2: field2_, comparison: comparison_ });
        }
    }
    (occurrences, relations)
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
            id: id_,
            name: fm_string_decrypt(name),
            filename: fm_string_decrypt(filename),
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

        let steps = Vec::<ScriptStep>::new();

        for step in code.chunks(28) {
            let mut args = Vec::<String>::new();
            let step_id = step[2];
            let instruction = INSTRUCTIONMAP[step[21] as usize].expect("Unknown script step.");
            let step_storage = match script_view.get_dir_relative(&mut HBAMPath::new(vec![&[5], &[step_id]])) {
                Some(inner) => inner,
                None => continue
            };


            match instruction {
                Instruction::SetVariable => {
                    let variable = step_storage
                        .get_dir_relative(&mut HBAMPath::new(vec![&[0, 0]])).unwrap();
                    let variable_name = fm_string_decrypt(variable.get_value(1).unwrap());

                    let expr_storage = step_storage
                        .get_dir_relative(&mut HBAMPath::new(vec![&[0, 1], &[5]])).unwrap();
                    let expr = expr_storage.get_value(5).unwrap();
                    let expr_decoded = decompile_calculation(expr);
                    args.push(variable_name);
                    args.push(expr_decoded);
                },
                _ => continue
            }
        }

        let created_by_ = script_view.get_value(64513).unwrap();
        let modified_by_ = script_view.get_value(64514).unwrap();

        let script = Script {
            name: name_,
            id: id_ as u16,
            instructions: steps,
            created_by: fm_string_decrypt(created_by_), 
            modified_by: fm_string_decrypt(modified_by_),
            arguments: vec![],
        };
        result.insert(id_, script);
    }

    result
}

pub fn get_layout_catalog(cache: &mut PageStore, file: &str) -> HashMap::<usize, LayoutFM> {
    let mut result = HashMap::new();
    let layout_view = match get_view_from_key(&HBAMPath::new(vec![&[4], &[1], &[7]]), cache, file).expect("Unable to read layout catalog") {
        Some(inner) => inner,
        None => { return result }
    };

    for layout in layout_view.get_dirs().unwrap() {
        let id_ = get_path_int(layout.path.components.last().unwrap());
        let meta_definition = layout.get_value(2).unwrap();
        let name_ = fm_string_decrypt(layout.get_value(16).unwrap());

        result.insert(id_, LayoutFM{
            id: id_,
            name: name_,
            table_occurrence: DBObjectReference {
                data_source: 0,
                top_id: meta_definition[1] as u16,
                inner_id: 0,
            },
            table_occurrence_name: String::new(),
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
    use crate::{hbam2::{api::get_occurrence_catalog, path::HBAMPath}, schema::TableOccurrence};
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

        let result = get_keyvalue(&key, &mut cache, "test_data/input/blank.fmp12").expect("Unable to get keyval");

        assert_eq!(result, Some(expected));
    }

    #[test]
    fn get_table_catalog_test() {
        let mut cache = PageStore::new();
        let result = get_table_catalog(&mut cache, "test_data/input/blank.fmp12");
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
        let result = get_table_catalog(&mut cache, "test_data/input/mixed.fmp12");
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
        let result = get_datasource_catalog(&mut cache, "test_data/input/mixed.fmp12");

        assert_eq!(2, result.len());

        for (_, ds) in result {
            println!("{} :: {:?}", ds.name, ds.filename);
        }
    }

    #[test]
    fn get_script_catalog_test() {
        let mut cache = PageStore::new();
        let result = get_script_catalog(&mut cache, "test_data/input/mixed.fmp12");

        assert_eq!(3, result.len());
    }

    #[test]
    fn get_table_occurrence_catalog_test() {
        let mut cache = PageStore::new();
        let (occurrences, relations) = get_occurrence_catalog(&mut cache, "test_data/input/relation.fmp12");
        assert_eq!(3, occurrences.len());
        assert_eq!(*occurrences.get(&129).unwrap(), TableOccurrence {
            id: 129,
            name: String::from("blank"),
            base_table: crate::schema::DBObjectReference { 
                data_source: 0, 
                top_id: 129, 
                inner_id: 0, 
            },
            created_by: String::from("admin"),
            modified_by: String::from("Admin"),
        });
        assert_eq!(*occurrences.get(&130).unwrap(), TableOccurrence {
            id: 130,
            name: String::from("blank 2"),
            base_table: crate::schema::DBObjectReference { 
                data_source: 0, 
                top_id: 129, 
                inner_id: 0, 
            },
            created_by: String::from("admin"),
            modified_by: String::from("Admin"),
        });
        assert_eq!(2, relations.len());
    }
}
