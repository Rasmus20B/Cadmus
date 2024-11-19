use crate::{hbam2::bplustree::get_view_from_key, schema::{AutoEntry, AutoEntryType, DBObjectReference, DataType, Field, LayoutFM, Relation, RelationComparison, RelationCriteria, Table, TableOccurrence, Validation, ValidationTrigger}, util::encoding_util::{fm_string_decrypt, get_path_int, put_path_int}};

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
            let id_ = get_path_int(&path_id);

            fields_.insert(id_, Field {
                id: id_,
                name: fm_string_decrypt(field.get_value(16).expect("Unable to get field name.")),
                dtype: DataType::Text,
                autoentry: AutoEntry { 
                    nomodify: false, 
                    definition: AutoEntryType::NA 
                },
                validation: Validation {
                    trigger: ValidationTrigger::OnEntry,
                    checks: vec![],
                    user_override: false,
                    message: String::from("Error with validation."),
                },
                global: false,
                repetitions: 1,
                created_by: fm_string_decrypt(field.get_value(64513).expect("Unable to get created by for field.")),
                modified_by: fm_string_decrypt(field.get_value(64514).expect("Unable to get modified by for field."))
            });
        }

        result.insert(id_ as usize, Table {
            id: id_ as usize,
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
            let mut tmp = Relation::new(def[4] as usize);
            tmp.table1 = DBObjectReference { 
                data_source: 0,
                top_id: id_ as u16,
                inner_id: 0,
            };
            tmp.table2 = DBObjectReference { 
                data_source: 0,
                top_id: def[2] as u16 + 128,
                inner_id: 0,
            };               

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
    match search_key(key, store, file) {
        Ok(inner) => Ok(inner),
        Err(e) => Err(e),
    }
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

    use super::{get_keyvalue, get_table_catalog, KeyValue, PageStore};
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
    fn get_table_occurrence_catalog_test() {
        let mut cache = PageStore::new();
        let (occurrences, relations) = get_occurrence_catalog(&mut cache, "test_data/input/relation.fmp12");
        assert_eq!(2, occurrences.len());
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
        assert_eq!(1, relations.len());
    }
}
