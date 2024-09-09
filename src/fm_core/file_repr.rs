use std::{borrow::Borrow, cell::RefCell, collections::HashMap, path::Path};

use crate::{fm_format::{self}, fm_io::handle::FmpFileHandle, util::encoding_util::fm_string_decrypt};
use super::component::FMTable;

pub struct FmpFileView {
    pub handle: RefCell<FmpFileHandle>,
    pub tables: HashMap::<usize, FMTable>,
}

impl FmpFileView {
    pub fn new(path: &Path) -> Self {
        let mut result = Self {
            handle: FmpFileHandle::from_path(path).into(),
            tables: HashMap::new(),
        };
        let handle_ = FmpFileHandle::from_path(path);
        let chunks = handle_.borrow().get_chunks();
        for c in chunks {
            fm_format::interpreter::read_chunk(&c, &mut result);
        }
        result
    }

    pub fn get_table_name(&self, index: usize) -> String {
        let table_ref = self.tables.get(&index).expect("table doesn't exist.");
        let table_name_ref = table_ref.table_name;

        let mut handle_ref = self.handle.borrow_mut();
        let chunk_ref = handle_ref.chunks[table_name_ref.chunk as usize].clone();
        let instr = chunk_ref.instructions[table_name_ref.block as usize].borrow();
        let name = handle_ref
            .fetch_data(table_name_ref);
        fm_string_decrypt(&name)
    }
}

