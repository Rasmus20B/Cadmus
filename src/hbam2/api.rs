use super::page_store::PageStore;


pub type Key = Vec<String>;
pub type Value = String;

#[derive(Debug, PartialEq, Eq)]
pub struct KeyValue {
    pub key: Vec<String>,
    pub value: Vec<u8>,
}

pub enum APIErr {
    KeyNotFound(Key)
}

pub fn get_keyvalue(key: Key, store: &PageStore) -> Result<KeyValue, APIErr> {
    // Get KeyVal pair from HBAM in byte form.
    unimplemented!()

}


pub fn get_data(key: Key) -> Result<KeyValue, APIErr> {
    unimplemented!()
}

pub fn set_keyvalue(key: Key, val: Value) -> Result<(), APIErr> {
    // setting a keyvalue to a Page
    // Process involes
    // 1. get a copy of the page from the block store
    // 2. set the key and check if there are chunks after it.
    // 3. If there are instructions after it, then split the page
    // 3.5 push page with new key, and new page with keys after the new one. 
    // 4. else just push the page as is to the cache.
    unimplemented!()
}
