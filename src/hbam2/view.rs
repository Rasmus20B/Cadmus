use super::{path::HBAMPath, chunk::{Chunk, ChunkContents}};

pub struct View<'a> {
    path: HBAMPath,
    chunks: Vec<Chunk<'a>>,
}

pub struct SubView<'a> {
    path: HBAMPath,
    chunks: Vec<&'a Chunk<'a>>,
}

pub enum KeyType {
    Simple(u16),
    Long(Vec<u8>),
}

impl<'a> SubView<'a> {
    pub fn new(path_: HBAMPath, chunks_: Vec<&'a Chunk<'a>>) -> Self {
        Self {
            path: path_,
            chunks: chunks_,
        }
    }

    pub fn get_value(&self, search: u16) -> Option<&[u8]> {
        for chunk in &self.chunks {
            match chunk.contents {
                ChunkContents::SimpleRef { key, data } => {
                    if search == key {
                        return Some(data)
                    }
                }
                _ => {}
            }
        }
        None
    }
}

impl<'a> View<'a> {
    pub fn new(path_: HBAMPath, chunks_: Vec<Chunk<'a>>) -> Self {
        Self {
            path: path_,
            chunks: chunks_,
        }
    }

    pub fn get_dirs(&self) -> Vec<SubView> {
        unimplemented!()
    }

    pub fn get_value(&self, search: u16) -> Option<&[u8]> {
        for chunk in &self.chunks {
            match chunk.contents {
                ChunkContents::SimpleRef { key, data } => {
                    if search == key {
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
            match chunk.contents {
                ChunkContents::Push { key } => {
                    current_dir.components.push(key.to_vec());
                }
                ChunkContents::Pop => {
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
