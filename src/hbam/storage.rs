use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BlockStorage {
    pub offset: u16,
    pub length: u16,
}

impl BlockStorage {
    pub fn lookup_from_buffer(&self, buffer: &[u8]) -> Result<Vec<u8>, &str> {
        Ok(buffer
            .get(self.offset as usize..self.offset as usize+self.length as usize)
            .unwrap()
            .to_vec())
    }
}
