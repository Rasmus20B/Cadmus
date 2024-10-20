use std::{fs::File, io::{Read, Seek}, path::Path, sync::{Arc, RwLock}};

use super::{api::{Key, KeyValue}, page::Page, page_store::PageStore};

pub(crate) enum BPlusTreeErr {
    KeyNotFound(Key),
    PageNotFound(usize)
}

pub fn get_page(index: usize) -> Arc<RwLock<Page>> {
    unimplemented!()
}

pub fn search_key(cache: &PageStore) -> Result<KeyValue, BPlusTreeErr> {
    unimplemented!()
}

fn load_page_from_disk(file: &Path, index: u64) -> Result<Page, BPlusTreeErr> {
    let mut buffer = [0u8; 4096];
    let mut handle = File::open(file)
        .expect("Unable to open file.");
    handle.seek(std::io::SeekFrom::Start(index * Page::PAGESIZE))
        .expect("Unable to seek into file.");
    handle.read_exact(&mut buffer)
        .expect("Unable to read from file.");

    Ok(Page::from_bytes(&buffer))


}
