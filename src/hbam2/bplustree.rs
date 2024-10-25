use std::{collections::HashMap, fs::File, io::{Read, Seek}, path::Path, sync::{Arc, RwLock}};

use crate::{hbam::{chunk::{BlockErr, Chunk, InstructionType}, path::HBAMPath}, util::encoding_util::{get_int, get_path_int}}; 

use super::{api::{Key, KeyValue}, page::{Page, PageHeader, PageType}, page_store::{PageStore, PageIndex}};

type Offset = usize;

#[derive(Debug)]
pub(crate) enum BPlusTreeErr {
    KeyNotFound(Key),
    EmptyKey,
    PageNotFound(usize),
    RootNotFound,
    InvalidChunkComposition(Chunk),
}

pub struct Cursor {
    key: Key,
    offset: Offset,
}

fn search_key_in_page(key_: &[String], page_data: &[u8; 4096]) -> Result<Option<KeyValue>, BPlusTreeErr> {
    if key_.is_empty() { return Ok(None); }
    let mut cursor = Cursor {
        key: vec![],
        offset: 20,
    };

    let mut path = vec![];
    while cursor.offset < page_data.len() {
        let chunk = match Chunk::from_bytes(page_data, &mut cursor.offset, &mut path) {
            Ok(inner) => inner,
            Err(BlockErr::EndChunk) => { continue; }
            _ => return Ok(None),
        };
        let key_without_suffix = &key_[0..key_.len() - 1];
        let suffix = &key_[key_.len() - 1];
        if chunk.path.components == key_without_suffix && chunk.ctype == InstructionType::RefSimple {
            if let Some(key) = chunk.ref_simple {
                if key.to_string() == *suffix {
                    return Ok(Some(KeyValue {
                        key: key_.to_vec(),
                        value: chunk.data.unwrap()
                            .lookup_from_buffer(page_data)
                            .expect("Unable to lookup data."),
                    }))
                }
            } else {
                return Err(BPlusTreeErr::InvalidChunkComposition(chunk))
            }
        }
    }
    Ok(None)
}

fn get_page(index: PageIndex, cache: &mut PageStore, file: &String) -> Result<Arc<Page>, BPlusTreeErr> {
    match cache.get(file.to_string(), index) {
        Some(inner) => Ok(inner),
        None => {
            if let Ok(page) = load_page_from_disk(Path::new(file), index) {
                cache.put(file.to_string(), index, &page);
                Ok(cache.get(file.to_string(), index).unwrap())
            } else {
                return Err(BPlusTreeErr::RootNotFound)
            }
        }
    }
}

fn search_index_page(key: &HBAMPath, page: Page) -> Result<PageIndex, BPlusTreeErr> {
    let mut cur_path = HBAMPath {
        components: vec![],
    };
    let mut offset = PageHeader::SIZE as usize;
    let search_key = HBAMPath::new(key.components[0..key.components.len() - 2].to_vec());
    let mut cur_index = 1;
    let mut delayed_pops = 0;
    while cur_path <= search_key  && offset < Page::SIZE as usize {
        if let Ok((chunk, delayed_pop)) = Chunk::from_bytes_new(&page.data, &mut(offset), &mut vec![]) {
            match chunk.ctype {
                InstructionType::PathPush => {
                    let data = chunk.data.unwrap().lookup_from_buffer(&page.data).unwrap();
                    let key_component = get_int(&chunk.data.unwrap().lookup_from_buffer(&page.data).unwrap()).to_string();

                    // FIXME: This is a total hack. a segment identifier seems to be known by a
                    // 4byte timestamp. This code turns that timestamp into a 0 to help with
                    // comparisons. This will NOT work when looking for 2 segments under the same
                    // path. I.e. 6.5.1.14.1347307366 == 6.5.1.14.1347309432 == 6.5.1.14.0.
                    if data.len() == 4 {
                        cur_path.components.push(String::from("0"));
                    } else {
                        cur_path.components.push(key_component.clone());
                    }
                    if data.len() == 1 {
                        if *key < cur_path {
                            return Ok(cur_index as u64);
                        }
                    }
                    if delayed_pop {
                        delayed_pops += 1;
                    }
                }
                InstructionType::PathPop => {
                    cur_path.components.pop();
                }
                InstructionType::RefSimple => {
                    cur_index = get_int(&chunk.data.unwrap().lookup_from_buffer(&page.data).unwrap());
                    cur_path.components.push(chunk.ref_simple.unwrap().to_string());
                    if *key <= cur_path {
                        return Ok(cur_index as u64);
                    }
                    cur_path.components.pop();
                    if delayed_pop {
                        delayed_pops += 1;
                        while delayed_pops > 0 {
                            cur_path.components.pop();
                            delayed_pops -= 1;
                        }
                    }
                }
                _ => {}
            }
        } else {
            return Err(BPlusTreeErr::KeyNotFound(key.components.to_vec()))
        }
    }
    return Err(BPlusTreeErr::KeyNotFound(key.components.to_vec()));
}

fn get_data_page(key: &Vec<String>, cache: &mut PageStore, file: &str) -> Result<Arc<Page>, BPlusTreeErr> {
    if key.is_empty() { return Err(BPlusTreeErr::EmptyKey) }
    // Get the root page
    // Follow the links through the index nodes, and subsequent index nodes. 
    // Once we get to the data node, we can use the next ptrs on the blocks.

    let root = get_page(1, cache, &file.to_string()).expect("Unable to get page from file.");

    let key = HBAMPath::new(key.to_vec());
    let mut cur_page = root;
    // TODO: detect page loops. I.e. page 64 -> 62 -> 64 -> 62
    loop {
        match cur_page.header.page_type {
            PageType::Data => {
                return Ok(cur_page);
            },
            PageType::Index | PageType::Root => {
                let next_index = search_index_page(&key, *cur_page);
                cur_page = get_page(next_index.unwrap(), cache, &file.to_string())?;
            }
        }
    }
}

pub fn search_key(key: &Vec<String>, cache: &mut PageStore, file: &str) -> Result<Option<KeyValue>, BPlusTreeErr> {
    if key.is_empty() { return Err(BPlusTreeErr::EmptyKey) }
    // Get the root page
    // Follow the links through the index nodes, and subsequent index nodes. 
    // Once we get to the data node, we can use the next ptrs on the blocks.

    let mut current_page = get_data_page(key, cache, file)?;
    loop {
        match search_key_in_page(&key, &current_page.data)? {
            Some(inner) => {
                return Ok(Some(inner))
            },
            None => {
                current_page = get_page(current_page.header.next as u64, cache, &file.to_string())?;
            }
        }
    }
}

pub fn load_page_from_disk(file: &Path, index: PageIndex) -> Result<Page, BPlusTreeErr> {
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
    use std::{fs::File, io::{BufReader, Read, Seek}, collections::hash_map::HashMap};

    use crate::{hbam2::{api::KeyValue, page::Page, page_store::PageStore}, HBAMPath};

    use super::{search_key_in_page, search_index_page};
 
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

        assert_eq!(Some(KeyValue {
            key: key,
            value: vec![3, 208, 0, 1],
        }), val);
    }

    #[test]
    fn index_page_parse_basic() {
        let file = File::open("test_data/input/blank.fmp12").expect("Unable to open file.");
        let mut reader = BufReader::new(&file);
        let mut buffer = [0u8; 4096];
        reader.seek(std::io::SeekFrom::Start(Page::SIZE)).expect("Unable to seek into the file.");
        reader.read_exact(&mut buffer).expect("Unable to read from file.");

        let page = Page::from_bytes(&buffer);

        let key = HBAMPath::new(vec![
            String::from("3"),
            String::from("17"),
            String::from("1"),
            String::from("0"),
        ]);

        let next_index = search_index_page(&key, page).expect("Unable to find next index from page.");
        assert_eq!(next_index, 64);
    }

    #[test]
    fn index_page_parse_layered_repeating_key_components_exact() {
        let file = File::open("test_data/input/blank.fmp12").expect("Unable to open file.");
        let mut reader = BufReader::new(&file);
        let mut buffer = [0u8; 4096];
        reader.seek(std::io::SeekFrom::Start(Page::SIZE)).expect("Unable to seek into the file.");
        reader.read_exact(&mut buffer).expect("Unable to read from file.");
        let page = Page::from_bytes(&buffer);
        let key = HBAMPath::new(vec![
            String::from("4"),
            String::from("5"),
            String::from("1"),
            String::from("13"),
            String::from("7"),
        ]);

        let next_index = search_index_page(&key, page).expect("Unable to find next index from page.");
        assert_eq!(next_index, 62);
    }

    #[test]
    fn index_page_parse_layered_repeating_key_components_after() {
        let file = File::open("test_data/input/blank.fmp12").expect("Unable to open file.");
        let mut reader = BufReader::new(&file);
        let mut buffer = [0u8; 4096];
        reader.seek(std::io::SeekFrom::Start(Page::SIZE)).expect("Unable to seek into the file.");
        reader.read_exact(&mut buffer).expect("Unable to read from file.");

        let page = Page::from_bytes(&buffer);

        let key = HBAMPath::new(vec![
            String::from("4"),
            String::from("5"),
            String::from("1"),
            String::from("13"),
            String::from("8"),
        ]);

        let next_index = search_index_page(&key, page).expect("Unable to find next index from page.");
        assert_eq!(next_index, 62);

    }

    #[test]
    fn index_page_parse_large_segment_list_start() {
        let file = File::open("test_data/input/blank.fmp12").expect("Unable to open file.");
        let mut reader = BufReader::new(&file);
        let mut buffer = [0u8; 4096];
        reader.seek(std::io::SeekFrom::Start(Page::SIZE)).expect("Unable to seek into the file.");
        reader.read_exact(&mut buffer).expect("Unable to read from file.");

        let page = Page::from_bytes(&buffer);

        let key = HBAMPath::new(vec![
            String::from("6"),
            String::from("5"),
            String::from("1"),
            String::from("14"),
            String::from("0"),
            String::from("1"),
        ]);

        let next_index = search_index_page(&key, page).expect("Unable to find next index from page.");
        assert_eq!(next_index, 61);
    }

    #[test]
    fn index_page_parse_large_segment_list_middle() {
        let file = File::open("test_data/input/blank.fmp12").expect("Unable to open file.");
        let mut reader = BufReader::new(&file);
        let mut buffer = [0u8; 4096];
        reader.seek(std::io::SeekFrom::Start(Page::SIZE)).expect("Unable to seek into the file.");
        reader.read_exact(&mut buffer).expect("Unable to read from file.");

        let page = Page::from_bytes(&buffer);

        let key = HBAMPath::new(vec![
            String::from("6"),
            String::from("5"),
            String::from("1"),
            String::from("14"),
            String::from("0"),
            String::from("59"),
        ]);

        let next_index = search_index_page(&key, page).expect("Unable to find next index from page.");
        assert_eq!(next_index, 28);

    }
}












