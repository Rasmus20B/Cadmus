use std::{borrow::BorrowMut, cell::{Cell, Ref, RefCell}, collections::HashMap, fs::File, io::{BufReader, BufWriter, Read, Seek}, ops::{Add, AddAssign}, path::Path, rc::Rc};
use super::block::{self, Block};

pub(crate) enum BlockCacheErr {

}

#[derive(Clone)]
pub struct Entry {
    file_name: String,
    pub block: Block,
    pub raw: [u8; 4096],
}


pub(crate) struct PageCache<const N: usize> {
    pages: [Option<(RefCell<usize>, Rc<RefCell<Entry>>)>; N],
    access_count: RefCell<usize>,
}

impl<const N: usize> PageCache<N> {
    pub fn get_block(&self, file_path: &str, index: usize) -> Result<Rc<RefCell<Entry>>, &str> {
        let mut blocks_iter = self.pages.iter();
        if let Some(cursor) = blocks_iter
            .find(|page| {
                if let Some(entry) = page {
                    let borrowed_entry = entry.1.borrow();
                    return borrowed_entry.block.index as usize == index;
                }
                false
            }) {
            self.access_count.borrow_mut().add_assign(1);
            let block = cursor.clone().unwrap().1;
            Ok(block)
        } else {
            let page_ = load_page_from_file(Path::new(file_path), index)
                .expect(&format!("Unable to get block {} from file.", index));
            let block_ = Block::new(&page_);
            let cursor = &mut blocks_iter
                .filter(|block| block.is_some())
                .min_by(|x, y| x.as_ref().unwrap().0
                    .cmp(&y.as_ref().unwrap().0));

            self.access_count.borrow_mut().add_assign(1);
            let entry = Rc::new(RefCell::new(Entry {
                file_name: file_path.to_string(),
                block: block_,
                raw: page_,
            }));
            let access_count = self.access_count.borrow().clone();

            match cursor {
                Some(inner) => {
                    let handle = inner.as_ref().unwrap();
                    *handle.0.borrow_mut() = access_count; 
                    *handle.1.clone().borrow_mut() = entry; 
                    Ok(inner.clone().unwrap().1)
                }
                None => {
                    let new_cursor = self.pages.iter().position(|entry| entry.is_none()).unwrap();
                    self.pages.[new_cursor] = Some((RefCell::new(access_count), entry));
                    Ok(self.pages[new_cursor].unwrap().1)
                }
            }
        }
    }


    pub fn create(index: usize) -> Result<(), BlockCacheErr> {
        unimplemented!()
    }

    pub fn delete(index: usize) -> Result<(), BlockCacheErr> {
        unimplemented!()
    }

    pub fn new() -> Self {
        Self {
            pages: [const {None}; N],
            access_count: 0.into(),
        }
    }
}

pub fn load_page_from_file(file_path: &Path, index: usize) -> Result<[u8; 4096], &str> {
    let mut buffer = [0u8; 4096];
    println!("Trying to open file: {:?}", file_path);
    let file = File::open(file_path).expect("Unable to open file.");
    let mut reader = BufReader::<File>::new(file);
    reader.seek(std::io::SeekFrom::Start(index as u64 * 4096))
        .expect("Could not seek into file.");
    reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");
    Ok(buffer)
}
