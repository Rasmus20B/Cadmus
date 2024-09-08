
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkOffset {
    pub chunk_index: u32,
    pub local_index: u32,
}

impl ChunkOffset {
    pub fn new(chunk_index_: u32, local_index_: u32) -> Self {
        Self {
            chunk_index: chunk_index_,
            local_index: local_index_,
        }
    }
}
