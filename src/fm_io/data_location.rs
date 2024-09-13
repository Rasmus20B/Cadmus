
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DataLocation {
    pub chunk: u32,
    pub block: u16,
    pub offset: u8,
    pub length: u8,
}

impl DataLocation {
    pub fn new(chunk_id_: u32, block_id_: u16, offset_: u8, length_: u8) -> Self {
        Self {
            chunk: chunk_id_,
            block: block_id_,
            offset: offset_,
            length: length_,
        }
    }
    pub fn fetch_data(&self, bytes: &[u8]) -> Result<Vec<u8>, &str> {
        let range = self.offset as usize..self.offset as usize +self.length as usize;
        Ok(bytes[range].to_vec())
    }
}
