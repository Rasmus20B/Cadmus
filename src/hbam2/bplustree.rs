use std::{cmp::Ordering, collections::{HashMap, HashSet}, fs::File, io::{Read, Seek}, path::Path, sync::{Arc, RwLock}};

use crate::util::encoding_util::{get_int, get_path_int}; 

use super::{api::{Key, KeyValue}, chunk::{Chunk, ChunkContents, LocalChunk, ParseErr}, page::{Page, PageHeader, PageType}, page_store::{PageIndex, PageStore}, path::HBAMPath, view::View};

type Offset = usize;

#[derive(Debug)]
pub(crate) enum BPlusTreeErr {
    KeyNotFound(Vec<Vec<u8>>),
    EmptyKey,
    PageNotFound(usize),
    RootNotFound,
    InvalidChunkComposition(ParseErr),
    PageCycle(PageIndex)
}

pub struct Cursor {
    key: Key,
    offset: Offset,
}

fn search_key_in_page<'a>(key_: &HBAMPath, page_data: &[u8; 4096]) -> Result<Option<KeyValue>, BPlusTreeErr> {
    if key_.components.is_empty() { return Ok(None); }
    let mut cursor = Cursor {
        key: vec![],
        offset: 20,
    };

    let mut key_without_suffix = key_.clone();
    key_without_suffix.components.pop();
    let suffix = key_.components[key_.components.len() - 1].clone();

    let suffix = if suffix.len() == 1 {
        suffix[0] as u16
    } else if suffix.len() == 2 {
        u16::from_be_bytes(suffix.into_iter().take(2).collect::<Vec<_>>().try_into().unwrap())
    } else {
        return Err(BPlusTreeErr::EmptyKey)
    };
    let mut path: HBAMPath = HBAMPath::new(vec![]); 
    while cursor.offset < page_data.len() {
        let chunk = match Chunk::from_bytes(page_data, &mut cursor.offset) {
            Ok(inner) => inner,
            Err(ParseErr::EndChunk) => { continue; }
            Err(e) => {
                return Err(BPlusTreeErr::InvalidChunkComposition(e))
            }
        };

        match &chunk.contents {
            ChunkContents::SimpleRef { key, data } => {
                if path == key_without_suffix
                    && *key == suffix {
                        return Ok(Some(KeyValue {
                            key: key_.components.to_vec(),
                            value: data.to_vec()
                        }))
                }
            },
            ChunkContents::Push { key } => {
                path.components.push(key.to_vec());
            },
            ChunkContents::Pop => {
                path.components.pop();
            },
            _ => {},
        }
    }
    Ok(None)
}

fn get_page<'a>(index: PageIndex, cache: &'a mut PageStore, file: &str) -> Result<Arc<Page>, BPlusTreeErr> {  
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

fn search_index_page(key_: &HBAMPath, page: Page) -> Result<PageIndex, BPlusTreeErr> {
    let mut cur_path = HBAMPath::new(vec![]);
    let mut offset = PageHeader::SIZE as usize;
    let search_key = key_.clone();
    let mut cur_index = 1;
    let mut delayed_pops = 0;

    while cur_path <= search_key  && offset < Page::SIZE as usize {
        if let Ok(chunk) = Chunk::from_bytes(&page.data, &mut(offset)) {
            match chunk.contents {
                ChunkContents::Push { key } => {
                    cur_path.components.push(key.to_vec());
                    if search_key < cur_path || (search_key == cur_path 
                        && search_key.components.len() == cur_path.components.len()) {
                        return Ok(cur_index as u64);
                    }
                    if chunk.delayed {
                        delayed_pops += 1;
                    }
                }
                ChunkContents::Pop => {
                    cur_path.components.pop();
                    if chunk.delayed {
                        delayed_pops += 1;
                    }
                }
                ChunkContents::SimpleData { .. } => {
                    if chunk.delayed {
                        delayed_pops += 1;
                    }
                }
                ChunkContents::SimpleRef { key, data } => {
                    cur_index = get_int(data);
                    cur_path.components.push(key.to_le_bytes().into());
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
            return Ok(cur_index as u64)
        }
    }
    return Err(BPlusTreeErr::KeyNotFound(key_.components.clone()));
}

fn get_data_page<'a, 'b>(key: &HBAMPath, cache: &'b mut PageStore, file: &str) -> Result<Arc<Page>, BPlusTreeErr> { 
    if key.components.is_empty() { return Err(BPlusTreeErr::EmptyKey) }
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
                let next_index = match search_index_page(&key, *cur_page) {
                    Ok(inner) => inner,
                    Err(BPlusTreeErr::KeyNotFound(_)) => cur_page.header.next as u64,
                    Err(e) => return Err(e)
                };
                if page_set.contains(&next_index) {
                    return Err(BPlusTreeErr::PageCycle(next_index))
                }
                page_set.insert(next_index.clone());
                cur_page = get_page(next_index, cache, file)?;
            }
        }
    }
}

pub fn get_view_from_key<'a, 'b>(key: &HBAMPath, cache: &mut PageStore, file: &str) -> Result<Option<View>, BPlusTreeErr> 
    where 'a: 'b
{
    if key.components.is_empty() { return Err(BPlusTreeErr::EmptyKey) }
    let mut current_page = get_data_page(&key, cache, file)?;
    let mut current_path = HBAMPath::new(vec![]);

    let mut chunks = vec![];
    let mut in_range = false;

    /* FIXME: This should search for the starting chunk. 
     * Then iterate until it reaches a chunk that is greater
     * than the search criteria. Right now it does not account for 
     * Views across multiple pages. */

    /* for split pages, take a copy of where we were in first page,
     * then skip pushes in next page until we reach the same path or 
     * get to a path that is after in the order. */

    let mut cached_path: HBAMPath;
    let mut offset = PageHeader::SIZE as usize;
    loop {
        while offset < Page::SIZE as usize {
            let chunk = match Chunk::from_bytes(&current_page.data, &mut offset) {
                Ok(inner) => inner,
                Err(ParseErr::EndChunk) => {
                    break;
                }
                Err(e) => return Err(BPlusTreeErr::InvalidChunkComposition(e)),
            };
            if current_path > *key {
                if chunks.is_empty() { return Ok(None) }
                else {
                    return Ok(Some(View::new(key.clone(), chunks)))
                }
            } else if current_path != *key && !chunks.is_empty() {
                return Ok(Some(View::new(key.clone(), chunks)))
            } else if current_path.components.len() >= key.components.len() && current_path == *key {
                chunks.push(chunk.copy_to_local())
            }

            match chunk.contents {
                ChunkContents::Push { key } => {
                    current_path.components.push(key.to_vec());
                },
                ChunkContents::Pop => {
                    current_path.components.pop();
                },
                _ => {}
            }
        }
        if current_page.header.next == 0 { return Ok(None) }
        cached_path = current_path.clone();
        current_path.components.clear();
        current_page = get_page(current_page.header.next.into(), cache, file)?;
        offset = PageHeader::SIZE as usize;
        while offset < Page::SIZE as usize
            && (current_path.components <= cached_path.components)
            && (current_path <= cached_path) {
            let chunk = match Chunk::from_bytes(&current_page.data, &mut offset) {
                Ok(inner) => inner,
                Err(ParseErr::EndChunk) => {
                    break;
                }
                Err(e) => return Err(BPlusTreeErr::InvalidChunkComposition(e)),
            };
            match chunk.contents {
                ChunkContents::Push { key } => {
                    current_path.components.push(key.to_vec());
                },
                ChunkContents::Pop => {
                    current_path.components.pop();
                },
                _ => {}
            }
        }
    }
}

pub fn search_key<'a>(key: &HBAMPath, cache: &'a mut PageStore, file: &'a str) -> Result<Option<KeyValue>, BPlusTreeErr> {
    if key.components.is_empty() { return Err(BPlusTreeErr::EmptyKey) }
    // Get the root page
    // Follow the links through the index nodes, and subsequent index nodes. 
    // Once we get to the data node, we can use the next ptrs on the blocks.

    let mut current_page = get_data_page(&key, cache, file)?;
    loop {
        match search_key_in_page(&key, &current_page.data)? {
            Some(inner) => {
                return Ok(Some(inner))
            },
            _ => {
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

pub fn print_tree(cache: &mut PageStore, file: &str) {
    let mut index = 1u32;
    let mut cursor = Cursor {
        key: vec![],
        offset: 20,
    };
    
    while index > 0 {
        let current_page = get_page(index as u64, cache, file).expect("Unable to get page");
        println!("Looking @ block {}", index);
        index = current_page.header.next;

        let mut path = HBAMPath::new(vec![]);

        while cursor.offset < Page::SIZE as usize {
            let chunk = Chunk::from_bytes(&current_page.data, &mut cursor.offset);
            let unwrapped = match chunk {
                Ok(inner) => inner,
                Err(ParseErr::EndChunk) => {continue;}
                Err(..) => {
                    return
                }
            };

            println!("{}::{}", path, unwrapped);

            match unwrapped.contents {
                ChunkContents::Push { key } => {
                    path.components.push(key.to_vec());
                }
                ChunkContents::Pop => {
                    path.components.pop();
                }
                _ => {}
            }
            if unwrapped.delayed {
                path.components.pop();
            }
        }

        cursor.offset = 20;
    }
}


#[cfg(test)]
mod tests {
    use std::{fs::File, io::{BufReader, Read, Seek}};
    use crate::hbam2::{api::KeyValue, page::Page, path::HBAMPath};
    use super::{search_key_in_page, search_index_page};
 
    #[test]
    fn get_keyvalue_test() {
        let file = File::open("test_data/input/blank.fmp12").expect("Unable to open file.");
        let mut reader = BufReader::new(&file);
        let mut buffer = [0u8; 4096];
        reader.seek(std::io::SeekFrom::Start(Page::SIZE * 64)).expect("Unable to seek into the file.");
        reader.read_exact(&mut buffer).expect("Unable to read from file.");

        let key = HBAMPath::new(vec![
            &[3], &[17], &[1], &[0]
        ]);
        let val = search_key_in_page(&key,
            &buffer).expect("Unable to find test key \"3, 17, 1, 0\" in blank file.");

        assert_eq!(Some(KeyValue {
            key: key.components,
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
            &[3], &[17], &[1], &[0]
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
            &[4], &[5], &[1], &[13], &[7]
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
            &[4], &[5], &[1], &[13], &[8]
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
            &[6], &[5], &[1], &[14], &[80, 78, 71, 102], &[1]
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
            &[6], &[5], &[1], &[14], &[80, 78, 71, 102], &[59]
        ]);

        let next_index = search_index_page(&key, page).expect("Unable to find next index from page.");
        assert_eq!(next_index, 28);

    }
}












