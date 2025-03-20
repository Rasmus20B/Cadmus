
mod frame;
mod page;
mod guard;

use std::sync::RwLockReadGuard;

use frame::Frame;
use page::{Page, PageGuard};


pub struct BufferPool {
    frames: Vec<Frame>,
    map: Vec<(u32, u32)>,
}

impl BufferPool {
    pub fn get_page(&mut self, page_id: u32) -> Option<RwLockReadGuard<Page>> {
        if let Some(key) = self.map.iter().map(|m| m.1).find(|needle| *needle == page_id) {
            if let Some(frame) = self.frames.get(key as usize) {
                return Some(frame.page.get_read())
            } 
        } 
        self._get_page_from_disk(page_id)
    }

    fn _get_page_from_disk(&mut self, page: u32) -> Option<RwLockReadGuard<Page>> {
        todo!()
    }
}

