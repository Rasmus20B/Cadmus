use std::{collections::HashMap, fs::File, io::{Read, Seek}, path::Path, sync::{Arc, RwLock}};

use crate::hbam::{chunk::{BlockErr, Chunk}, path::HBAMPath};

use super::{api::{Key, KeyValue}, page::{Page, PageHeader}, page_store::PageStore};

pub(crate) enum BPlusTreeErr {
    KeyNotFound(Key),
    EmptyKey,
    PageNotFound(usize),
    RootNotFound,
}

pub struct Cursor {
    key: Key,
    offset: usize,
}

fn search_key_in_page(key_: &[String], page_data: &[u8; 4096]) -> Option<KeyValue> {

    if key_.is_empty() { return None; }
    let mut cursor = Cursor {
        key: vec![],
        offset: 20,
    };

    let mut path = vec![];
    while cursor.offset < page_data.len() {
        let chunk = match Chunk::from_bytes(page_data, &mut cursor.offset, &mut path) {
            Ok(inner) => inner,
            Err(BlockErr::EndChunk) => { continue; }
            _ => return None,
        };
        if chunk.path.components == key_[0..key_.len()-1] {
            if chunk.ref_simple.unwrap_or(u16::max_value()).to_string() == key_[key_.len() - 1] {
                return Some(KeyValue {
                    key: key_.to_vec(),
                    value: chunk.data.unwrap()
                        .lookup_from_buffer(page_data)
                        .expect("Unable to lookup data."),
                })
            }
        }
    }
    None
}

pub fn search_key(key: &Vec<String>, cache: &mut PageStore, file_map: &HashMap<String, String>) -> Result<KeyValue, BPlusTreeErr> {
    if key.is_empty() { return Err(BPlusTreeErr::EmptyKey) }
    // Get the root page
    // Follow the links through the index node, and subsequent index nodes. 
    // One we get to the data node, we can use the next ptrs on the blocks.
    let root = match cache.get_root() {
        Some(inner) => inner,
        None => {
            if let Ok(page) = load_page_from_disk(Path::new(&file_map[&key[0]]), 1) {
                cache.put(1, &page);
                cache.get_root().unwrap()
            } else {
                return Err(BPlusTreeErr::RootNotFound)
            }
        }
    };

    unimplemented!()
}

pub fn load_page_from_disk(file: &Path, index: u64) -> Result<Page, BPlusTreeErr> {
    let mut buffer = [0u8; 4096];
    let mut handle = File::open(file)
        .expect("Unable to open file.");
    handle.seek(std::io::SeekFrom::Start(index * Page::SIZE))
        .expect("Unable to seek into file.");
    handle.read_exact(&mut buffer)
        .expect("Unable to read from file.");

    Ok(Page::from_bytes(&buffer))
}


#[cfg(test)]
mod tests {
    use std::{fs::File, io::{BufReader, Read, Seek}};

    use crate::hbam2::{api::KeyValue, page::Page};

    use super::search_key_in_page;
 
    #[test]
    fn get_keyvalue_test() {
        let file = File::open("test_data/input/blank.fmp12").expect("Unable to open file.");
        let mut reader = BufReader::new(&file);
        let mut buffer = [0u8; 4096];
        reader.seek(std::io::SeekFrom::Start(Page::SIZE * 64)).expect("Unable to seek into the file.");
        reader.read_exact(&mut buffer).expect("Unable to read from file.");

        let key = vec![
            String::from("3"),
            String::from("17"),
            String::from("1"),
            String::from("0"),
        ];
        let val = search_key_in_page(&key,
            &buffer).expect("Unable to find test key \"3, 17, 1, 0\" in blank file.");

        assert_eq!(KeyValue {
            key: key,
            value: vec![3, 208, 0, 1],
        }, val);
    }
}












