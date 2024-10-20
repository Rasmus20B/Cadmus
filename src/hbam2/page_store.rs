use std::{collections::HashMap, sync::Arc};

use super::{api::Key, page::Page};


pub struct PageStore {
    store: HashMap<u64, Arc<Page>>,
}

impl PageStore {
    pub fn get(&self, page: u64) -> Option<Arc<Page>> {
        self.store.get(&page).cloned()
    }

    fn put(&mut self, index: u64, page: &Page) {
        let mut handle = self.store.get_mut(&index);
        if let Some(ref mut unwrapped) = handle {
            *unwrapped = &mut Arc::new(*page);
        } else {
            self.store.insert(index, Arc::new(*page));
        }
    }
}
