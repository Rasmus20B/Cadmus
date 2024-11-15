use std::{collections::{HashMap, HashSet}, fs::File, io::{Read, Seek}, path::Path, sync::{Arc, RwLock}};

use crate::util::encoding_util::{get_int, get_path_int}; 

use super::{api::{Key, KeyValue}, view::View, path::HBAMPath, chunk::{Chunk, ChunkContents, ParseErr}, page::{Page, PageHeader, PageType}, page_store::{PageIndex, PageStore}};

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
    //println!("Current: {:?}, search: {:?}", cur_path, search_key);
    while cur_path <= search_key  && offset < Page::SIZE as usize {
        if let Ok(chunk) = Chunk::from_bytes(&page.data, &mut(offset)) {
            match chunk.contents {
                ChunkContents::Push { key } => {
                    // FIXME: This is a total hack. a segment identifier seems to be known by a
                    // 4byte timestamp. This code turns that timestamp into a 0 to help with
                    // comparisons. This will NOT work when looking for 2 segments under the same
                    // path. I.e. 6.5.1.14.1347307366 == 6.5.1.14.1347309432 == 6.5.1.14.0.
                    if key.len() >= 4 {
                        cur_path.components.push([0].to_vec());
                    } else {
                        cur_path.components.push(key.to_vec());
                    }
                    if key.len() == 1 && search_key < cur_path {
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
            return Err(BPlusTreeErr::KeyNotFound(key_.components.to_vec()))
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
                let next_index = search_index_page(&key, *cur_page)?;
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

    loop {
        let mut offset = 20usize;
        while offset < Page::SIZE as usize {
            let chunk = Chunk::from_bytes(&current_page.data, &mut offset).expect("Unable to decode chunk.");

            if key.contains(&current_path) {
                chunks.push(chunk.copy_to_local())
            } else if current_path > *key {
                if chunks.is_empty() { return Ok(None) }
                else {
                    return Ok(Some(View::new(key.clone(), chunks)))
                }
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
        current_page = get_page(current_page.header.next.into(), cache, file)?;
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
            &[6], &[5], &[1], &[14], &[0], &[1]
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
            &[6], &[5], &[1], &[14], &[0], &[59]
        ]);

        let next_index = search_index_page(&key, page).expect("Unable to find next index from page.");
        assert_eq!(next_index, 28);

    }
}












