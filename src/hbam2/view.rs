use super::{path::HBAMPath, chunk::{Chunk, LocalChunk, LocalChunkContents, ChunkContents}};

#[derive(Debug)]
pub struct View {
    path: HBAMPath,
    chunks: Vec<LocalChunk>,
}

#[derive(Debug)]
pub struct SubView<'a> {
    pub path: HBAMPath,
    chunks: Vec<&'a LocalChunk>,
}

pub enum KeyType {
    Simple(u16),
    Long(Vec<u8>),
}

impl<'a> SubView<'a> {
    pub fn new(path_: HBAMPath, chunks_: Vec<&'a LocalChunk>) -> Self {
        Self {
            path: path_,
            chunks: chunks_,
        }
    }

    pub fn get_value(&self, search: u16) -> Option<&[u8]> {
        for chunk in &self.chunks {
            match &chunk.contents {
                LocalChunkContents::SimpleRef { ref key, data } => {
                    if search == *key {
                        return Some(data)
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn get_dirs(&self) -> Option<Vec<SubView>> {
        let mut depth = 0;
        let mut result = vec![];
        let mut current_collection = std::hint::black_box(vec![]);
        let mut current_path = HBAMPath::new(self.path.components.iter().map(|c| c.as_slice()).collect());
        for chunk in self.chunks.iter().skip(1) {
            match &chunk.contents {
                LocalChunkContents::Push { key } => {
                    current_collection.push(*chunk);
                    current_path.components.push(key.to_vec());
                    depth += 1;
                }
                LocalChunkContents::Pop => {
                    current_collection.push(chunk);
                    depth -= 1;
                    if depth == 0 {
                        result.push( SubView::new(
                                current_path.clone(),
                                current_collection.clone(),
                                ) 
                        );
                        current_collection.clear();
                    }
                    current_path.components.pop();
                }
                _ => {
                    if current_path.components.len() <= self.path.components.len() {
                        continue;
                    }
                    current_collection.push(chunk);
                }
            }
        }

        if result.is_empty() { return None }
        else {
            return Some(result)
        }
    }
}

impl<'a> View {
    pub fn new(path_: HBAMPath, chunks_: Vec<LocalChunk>) -> Self {
        Self {
            path: path_,
            chunks: chunks_,
        }
    }

    pub fn get_dirs(&self) -> Option<Vec<SubView>> {
        let mut depth = 0;
        let mut result = vec![];
        let mut current_collection = std::hint::black_box(vec![]);
        let mut current_path = HBAMPath::new(self.path.components.iter().map(|c| c.as_slice()).collect());
        for chunk in &self.chunks {
            //println!("{:?}", current_path);
            match &chunk.contents {
                LocalChunkContents::Push { key } => {
                    current_collection.push(chunk);
                    current_path.components.push(key.to_vec());
                    depth += 1;
                }
                LocalChunkContents::Pop => {
                    current_collection.push(chunk);
                    depth -= 1;
                    if depth == 0 {
                        result.push( SubView::new(
                                current_path.clone(),
                                current_collection.clone(),
                                ) 
                        );
                        current_collection.clear();
                    }
                    current_path.components.pop();
                }
                _ => {
                    current_collection.push(chunk);
                }
            }
        }

        if result.is_empty() { return None }
        else {
            return Some(result)
        }
    }

    pub fn get_simple(&self, search: u16) -> Option<Vec<&[u8]>> {
        unimplemented!()
    }

    pub fn get_value(&self, search: u16) -> Option<&[u8]> {
        for chunk in &self.chunks {
            match &chunk.contents {
                LocalChunkContents::SimpleRef { key, data } => {
                    if search == *key {
                        return Some(data)
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn get_dir<'b>(&'a self, search: HBAMPath) -> Option<SubView<'b>> where 'a:'b {
        let mut current_dir = self.path.clone();

        let mut result_chunks = vec![];
        for chunk in &self.chunks {
            match &chunk.contents {
                LocalChunkContents::Push { key } => {
                    current_dir.components.push(key.to_vec());
                }
                LocalChunkContents::Pop => {
                    current_dir.components.pop();
                }
                _ => { }
            }
            if current_dir > search {
                if result_chunks.is_empty() { return None }
                else {
                    return Some(SubView::new(search, result_chunks))
                }
            }
            if current_dir == search {
               result_chunks.push(chunk)
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::HBAMPath;
    use std::io::{Read, Seek};
    use crate::{BufReader, File};
    use crate::hbam2::{bplustree::get_view_from_key, page::Page, page_store::PageStore};

    #[test]
    fn dirs_test() {
        let file = File::open("test_data/input/blank.fmp12").expect("Unable to open file.");
        let mut reader = BufReader::new(&file);
        let mut buffer = [0u8; 4096];
        reader.seek(std::io::SeekFrom::Start(Page::SIZE * 64)).expect("Unable to seek into the file.");
        reader.read_exact(&mut buffer).expect("Unable to read from file.");
        let mut cache = PageStore::new();
        let key = HBAMPath::new(vec![
            &[3], &[17], &[1]
        ]);
        let val = get_view_from_key(
            &key,
            &mut cache,
            "test_data/input/blank.fmp12").expect("Unable to find test key \"3, 17, 1, 0\" in blank file.").unwrap();
        assert_eq!(56, val.chunks.len());

        let dirs = val.get_dirs().unwrap();
        assert_eq!(dirs[0].path, HBAMPath::new(vec![
            &[3], &[17], &[1], &[1]
        ]));
        assert_eq!(dirs[1].path, HBAMPath::new(vec![
            &[3], &[17], &[1], &[3]
        ]));
        assert_eq!(dirs[2].path, HBAMPath::new(vec![
            &[3], &[17], &[1], &[8]
        ]));
        assert_eq!(dirs[3].path, HBAMPath::new(vec![
            &[3], &[17], &[1], &[14]
        ]));

        assert_eq!(val.get_value(0).unwrap(), vec![3, 208, 0, 1]);
        assert_eq!(val.get_value(64514).unwrap(), vec![27, 62, 55, 51, 52]);
    }

    #[test]
    fn subdirs_test() {
        let file = File::open("test_data/input/blank.fmp12").expect("Unable to open file.");
        let mut reader = BufReader::new(&file);
        let mut buffer = [0u8; 4096];
        reader.seek(std::io::SeekFrom::Start(Page::SIZE * 64)).expect("Unable to seek into the file.");
        reader.read_exact(&mut buffer).expect("Unable to read from file.");
        let mut cache = PageStore::new();
        let key = HBAMPath::new(vec![
            &[3], &[17], &[1]
        ]);
        let val = get_view_from_key(
            &key,
            &mut cache,
            "test_data/input/blank.fmp12").expect("Unable to find test key \"3, 17, 1, 0\" in blank file.").unwrap();

        let dirs = val.get_dirs().unwrap();

        let subdirs_of_14 = dirs[3].get_dirs().unwrap();
        let subdirs_of_129 = subdirs_of_14[0].get_dirs().unwrap();
        assert_eq!(subdirs_of_14[0].path, HBAMPath::new(vec![
            &[3], &[17], &[1], &[14], &[128, 1]
        ]));
        assert_eq!(subdirs_of_129[0].path, HBAMPath::new(vec![
            &[3], &[17], &[1], &[14], &[128, 1], &[255]
        ]));
        assert_eq!(subdirs_of_129[1].path, HBAMPath::new(vec![
            &[3], &[17], &[1], &[14], &[128, 1], &[255, 0]
        ]));
        assert_eq!(subdirs_of_129[2].path, HBAMPath::new(vec![
            &[3], &[17], &[1], &[14], &[128, 1], &[255, 2]
        ]));
        assert_eq!(subdirs_of_129[3].path, HBAMPath::new(vec![
            &[3], &[17], &[1], &[14], &[128, 1], &[255, 252]
        ]));
        assert_eq!(subdirs_of_129[0].get_value(1).unwrap(), vec![1, 1]);
        assert_eq!(subdirs_of_129[0].get_value(2).unwrap(), vec![1, 2]);
        assert_eq!(subdirs_of_129[0].get_value(3).unwrap(), vec![1, 3]);
        assert_eq!(subdirs_of_129[0].get_value(4).unwrap(), vec![1, 4]);
        assert_eq!(subdirs_of_129[0].get_value(5).unwrap(), vec![1, 5]);

        assert_eq!(subdirs_of_129[1].get_value(2).unwrap(), vec![1, 1, 2, 1, 1]);
        assert_eq!(subdirs_of_129[1].get_value(3).unwrap(), vec![1, 1, 2, 1, 2]);
        assert_eq!(subdirs_of_129[1].get_value(4).unwrap(), vec![1, 1, 2, 1, 3]);
        assert_eq!(subdirs_of_129[1].get_value(5).unwrap(), vec![1, 1, 2, 1, 4]);
        assert_eq!(subdirs_of_129[1].get_value(6).unwrap(), vec![1, 1, 2, 1, 5]);
    }
}
