use std::{collections::HashMap, sync::atomic::AtomicUsize};
use super::{frame::Frame, page_guard::{PageReadGuard, PageWriteGuard}};

pub struct BufferPoolManager {
    num_frames: usize,
    next_page_id: AtomicUsize,
    page_table: HashMap<usize, usize>,
    frames: Vec<Frame>,
}

impl BufferPoolManager {
    pub fn new(num_frames_: usize, k_dist_: usize) -> Self {
        todo!()
    }

    pub fn new_page() -> usize { todo!() }

    pub fn delete_page() -> usize { todo!() }

    pub fn checked_write_page(page_id: usize) -> Option<PageReadGuard> { todo!() }

    pub fn write_page(page_id: usize) -> PageWriteGuard { todo!() }

    pub fn read_page(page_id: usize) -> PageReadGuard { todo!() }

    pub fn flush_page(page_id: usize) -> bool { todo!() }

    pub fn get_pin_count(page_id: usize) -> Option<usize> { todo!() }
}
