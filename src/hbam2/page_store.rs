use std::{collections::hash_map::HashMap, sync::Arc};

use super::{api::Key, bplustree::load_page_from_disk, page::Page};

pub type FileIndex = u8;
pub type PageIndex = u64;

pub struct PageStore {
    file_map: HashMap<String, FileIndex>,
    store: HashMap<(FileIndex, PageIndex), Arc<Page>>,
}

impl PageStore {
    pub fn new() -> Self {
        Self {
            file_map: HashMap::new(),
            store: HashMap::new(),
        }
    }

    pub fn get(&self, file: String, page: PageIndex) -> Option<Arc<Page>> {
        match self.file_map.get(&file) {
            Some(handle) => self.store.get(&(*handle, page)).cloned(),
            None => None 
        }
    }

    pub fn get_root(&self, file: String) -> Option<Arc<Page>> {
        self.get(file, 1)
    }

    pub fn put(&mut self, file: String, index: u64, page: &Page) {
        let file_map_len = self.file_map.len();
        let file_index = *self.file_map.entry(file).or_insert_with_key(|_| file_map_len as u8);
        let mut handle = self.store.get_mut(&(file_index, index));
        if let Some(ref mut unwrapped) = handle {
            *unwrapped = &mut Arc::new(*page);
        } else {
            self.store.insert((file_index, index), Arc::new(*page));
        }
    }
}
