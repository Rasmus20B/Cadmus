use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};


pub type Page = [u8; 4096];

pub struct PageGuard {
    page: Arc<RwLock<Page>>,
    dirty: bool,
}

impl PageGuard {
    pub fn get_write(&self) -> RwLockWriteGuard<Page> {
        self.page.write().unwrap()
    }
    pub fn get_read(&self) -> RwLockReadGuard<Page> {
        self.page.read().unwrap()
    }
}
