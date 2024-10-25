use std::{collections::{HashMap, HashSet}, fs::File, io::{Read, Seek}, path::Path, sync::{Arc, RwLock}};

use crate::{hbam::path::HBAMPath, util::encoding_util::{get_int, get_path_int}}; 

use super::{api::{Key, KeyValue}, chunk::{Chunk, ChunkContents, ParseErr}, page::{Page, PageHeader, PageType}, page_store::{PageIndex, PageStore}};

type Offset = usize;

#[derive(Debug)]
pub(crate) enum BPlusTreeErr<'a> {
    KeyNotFound(Key),
    EmptyKey,
    PageNotFound(usize),
    RootNotFound,
    InvalidChunkComposition(ParseErr, Option<Chunk<'a>>),
    PageCycle(PageIndex)
}

pub struct Cursor {
    key: Key,
    offset: Offset,
}

fn search_key_in_page<'a>(key_: &'a [String], page_data: &[u8; 4096]) -> Result<Option<KeyValue>, BPlusTreeErr<'a>> {
    if key_.is_empty() { return Ok(None); }
    let mut cursor = Cursor {
        key: vec![],
        offset: 20,
    };

    let key_without_suffix = &key_[0..key_.len() - 1];
    let suffix = &key_[key_.len() - 1];
    let mut path: Vec<String> = vec![];
    while cursor.offset < page_data.len() {
        let chunk = match Chunk::from_bytes(page_data, &mut cursor.offset) {
            Ok(inner) => inner,
            Err(ParseErr::EndChunk) => { continue; }
            Err(e) => {
                return Err(BPlusTreeErr::InvalidChunkComposition(e, None))
            }
        };

        match &chunk.contents {
            ChunkContents::SimpleRef { key, data } => {
                if path == key_without_suffix
                    && key.to_string() == *suffix {
                        return Ok(Some(KeyValue {
                            key: key_.to_vec(),
                            value: data.to_vec()
                        }))
                }
            },
            ChunkContents::Push { key } => {
                path.push(get_int(key).to_string());
            },
            ChunkContents::Pop => {
                path.pop();
            },
            _ => {},
        }
    }
    Ok(None)
}

fn get_page<'a, 'b>(index: PageIndex, cache: &'b mut PageStore, file: &'a str) -> Result<Arc<Page>, BPlusTreeErr<'a>> where 'a: 'b {  
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

fn search_index_page(key_: &[String], page: Page) -> Result<PageIndex, BPlusTreeErr<'_>> {
    let mut cur_path = HBAMPath {
        components: vec![],
    };
    let mut offset = PageHeader::SIZE as usize;
    let search_key = HBAMPath::new(key_.to_vec());
    let mut cur_index = 1;
    let mut delayed_pops = 0;
    while cur_path <= search_key  && offset < Page::SIZE as usize {
        if let Ok(chunk) = Chunk::from_bytes(&page.data, &mut(offset)) {
            match chunk.contents {
                ChunkContents::Push { key } => {
                    let data = key;
                    let key_component = get_int(key).to_string();

                    // FIXME: This is a total hack. a segment identifier seems to be known by a
                    // 4byte timestamp. This code turns that timestamp into a 0 to help with
                    // comparisons. This will NOT work when looking for 2 segments under the same
                    // path. I.e. 6.5.1.14.1347307366 == 6.5.1.14.1347309432 == 6.5.1.14.0.
                    if data.len() == 4 {
                        cur_path.components.push(String::from("0"));
                    } else {
                        cur_path.components.push(key_component.clone());
                    }
                    if data.len() == 1 && search_key < cur_path {
                        return Ok(cur_index as u64);
                    }
                    if chunk.delayed {
                        delayed_pops += 1;
                    }
                }
                ChunkContents::Pop => {
                    cur_path.components.pop();
                }
                ChunkContents::SimpleRef { key, data } => {
                    cur_index = get_int(data);
                    cur_path.components.push(key.to_string());
                    if search_key <= cur_path {
                        return Ok(cur_index as u64);
                    }
                    cur_path.components.pop();
                    if chunk.delayed {
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
            return Err(BPlusTreeErr::KeyNotFound(key_.to_vec()))
        }
    }
    return Err(BPlusTreeErr::KeyNotFound(key_.to_vec()));
}

fn get_data_page<'a, 'b>(key: &'a [String], cache: &'b mut PageStore, file: &'a str) -> Result<Arc<Page>, BPlusTreeErr<'a>> where 'a: 'b {
    if key.is_empty() { return Err(BPlusTreeErr::EmptyKey) }
    // Get the root page
    // Follow the links through the index nodes, and subsequent index nodes. 
    // Once we get to the data node, we can use the next ptrs on the blocks.

    let mut page_set = HashSet::new();
    let root = get_page(1, cache, &file.to_string()).expect("Unable to get page from file.");
    let mut cur_page = root;
    loop {
        match cur_page.header.page_type {
            PageType::Data => {
                return Ok(cur_page);
            },
            PageType::Index | PageType::Root => {
                let next_index = search_index_page(key, *cur_page)?;
                if page_set.contains(&next_index) {
                    return Err(BPlusTreeErr::PageCycle(next_index))
                }
                page_set.insert(next_index.clone());
                cur_page = get_page(next_index, cache, file)?;
            }
        }
    }
}

pub fn search_key<'a, 'b>(key: &'a [String], cache: &'b mut PageStore, file: &'a str) -> Result<Option<KeyValue>, BPlusTreeErr<'b>> where 'a: 'b {
    if key.is_empty() { return Err(BPlusTreeErr::EmptyKey) }
    // Get the root page
    // Follow the links through the index nodes, and subsequent index nodes. 
    // Once we get to the data node, we can use the next ptrs on the blocks.

    let mut current_page = get_data_page(key, cache, file)?;
    loop {
        match search_key_in_page(key, &current_page.data)? {
            Some(inner) => {
                return Ok(Some(inner))
            },
            None => {
                current_page = get_page(current_page.header.next as u64, cache, &file)?;
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
    use std::{fs::File, io::{BufReader, Read, Seek}};
    use crate::hbam2::{api::KeyValue, page::Page};
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

        let key = vec![
            String::from("3"),
            String::from("17"),
            String::from("1"),
            String::from("0"),
        ];

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
        let key = vec![
            String::from("4"),
            String::from("5"),
            String::from("1"),
            String::from("13"),
            String::from("7"),
        ];

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

        let key = vec![
            String::from("4"),
            String::from("5"),
            String::from("1"),
            String::from("13"),
            String::from("8"),
        ];

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

        let key = vec![
            String::from("6"),
            String::from("5"),
            String::from("1"),
            String::from("14"),
            String::from("0"),
            String::from("1"),
        ];

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

        let key = vec![
            String::from("6"),
            String::from("5"),
            String::from("1"),
            String::from("14"),
            String::from("0"),
            String::from("59"),
        ];

        let next_index = search_index_page(&key, page).expect("Unable to find next index from page.");
        assert_eq!(next_index, 28);

    }
}












