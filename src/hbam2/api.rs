use super::{page_store::PageStore, bplustree::{search_key, BPlusTreeErr}};

use std::collections::hash_map::HashMap;

pub type Key = Vec<String>;
pub type Value = String;

#[derive(Debug, PartialEq, Eq)]
pub struct KeyValue {
    pub key: Vec<String>,
    pub value: Vec<u8>,
}

pub fn get_keyvalue<'a>(key: &'a Key, store: &'a mut PageStore, file: &'a str) -> Result<Option<KeyValue>, BPlusTreeErr<'a>> {
    // Get KeyVal pair from HBAM in byte form.
    match search_key(key, store, file) {
        Ok(inner) => Ok(inner),
        Err(e) => Err(e),
    }
}


pub fn get_data<'a>(key: Key) -> Result<KeyValue, BPlusTreeErr<'a>> {
    unimplemented!()
}

pub fn set_keyvalue<'a>(key: Key, val: Value) -> Result<(), BPlusTreeErr<'a>> {
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

    use super::{KeyValue, get_keyvalue, PageStore};
    #[test]
    fn get_keyval_test() {

        let key_ = vec![
            String::from("3"),
            String::from("17"),
            String::from("1"),
            String::from("0"),
        ];
        let expected = KeyValue {
            key: key_.clone(),
            value: vec![3, 208, 0, 1],
        };

        let mut cache = PageStore::new();

        let result = get_keyvalue(&key_, &mut cache, "test_data/input/blank.fmp12").expect("Unable to get keyval");

        assert_eq!(result, Some(expected));
    }


}
