use std::{borrow::Borrow, cell::RefCell, collections::HashMap, path::Path, sync::{Arc, Mutex, RwLock}};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{fm_format::{self}, fm_io::handle::FmpFileHandle, util::encoding_util::fm_string_decrypt};
use super::component::FMTable;

pub struct FmpFileView {
    pub handle: RefCell<FmpFileHandle>,
    pub objects: RwLock<FmpObjectManager>,
}

pub struct FmpObjectManager {
    pub tables: HashMap::<usize, FMTable>,
}

impl FmpObjectManager {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new()
        }
    }
}

impl FmpFileView {
    pub fn new(path: &Path) -> Self {
        let handle_ = FmpFileHandle::from_path(path);
        let objects_ = RwLock::new(FmpObjectManager::new());
        let chunks = handle_.borrow().get_chunks();
        chunks.par_iter()
            .for_each(|c| fm_format::interpreter::read_chunk(&c, &objects_));

        Self {
            handle: handle_.into(),
            objects: objects_
        }
    }

    pub fn get_table_name(&self, index: usize) -> String {
        let objects_ref = self.objects.read().expect("Unable to acquire read lock to objects.");
        let table_ref = objects_ref.tables.get(&index).expect("table doesn't exist.");
        let table_name_ref = table_ref.table_name;

        let mut handle_ref = self.handle.borrow_mut();
        let block_ref = handle_ref.blocks[table_name_ref.chunk as usize].clone();
        let instr = block_ref.chunks[table_name_ref.block as usize].borrow();
        let name = handle_ref
            .fetch_data(table_name_ref);
        fm_string_decrypt(&name)
    }
}

