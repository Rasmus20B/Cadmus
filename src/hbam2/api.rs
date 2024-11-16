use crate::{hbam2::bplustree::get_view_from_key, schema::Table, util::encoding_util::{fm_string_decrypt, get_path_int}};

use super::{page_store::PageStore, path::HBAMPath, bplustree::{search_key, BPlusTreeErr}};

use std::collections::hash_map::HashMap;

pub type Key = Vec<String>;
pub type Value = String;

#[derive(Debug, PartialEq, Eq)]
pub struct KeyValue {
    pub key: Vec<Vec<u8>>,
    pub value: Vec<u8>,
}

pub fn get_table_catalog(cache: &mut PageStore, file: &str) -> HashMap<usize, Table> {
    let view_option = match get_view_from_key(&HBAMPath::new(vec![&[3], &[16], &[5]]), cache, file)
        .expect("Unable to get table info from file.") {
            Some(inner) => inner,
            None => return HashMap::new()
        };

    let mut result = HashMap::new();
    for dir in view_option.get_dirs().unwrap() {
        let id_ = get_path_int(&dir.path.components[dir.path.components.len()-1]);
        result.insert(id_ as usize, Table {
            id: id_ as usize,
            name: fm_string_decrypt(dir.get_value(16).unwrap()),
            created_by: fm_string_decrypt(dir.get_value(64513).unwrap()),
            modified_by: fm_string_decrypt(dir.get_value(64513).unwrap()),
            fields: HashMap::new(),
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
    use crate::hbam2::path::HBAMPath;
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
        }
    }
}
