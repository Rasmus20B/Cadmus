use std::path::Path;

use super::page_store::PageStore;


pub struct Context {
    cache: PageStore,
}

impl Context {
    pub fn new() -> Self {
        Self {
            cache: PageStore::new(),
        }
    }

    pub fn get_table_catalog() {
        unimplemented!()
    }
}
