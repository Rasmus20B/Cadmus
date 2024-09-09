
#[derive(Debug, Clone, Copy)]
pub struct BlockStorage {
    pub offset: u32,
    pub length: u16,
}

impl BlockStorage {
    pub fn lookup_from_buffer(&self, buffer: &Vec<u8>) -> Result<Vec<u8>, &str> {
        Ok(buffer
            .get(self.offset as usize..self.offset as usize+self.length as usize)
            .unwrap()
            .to_vec())
    }
}
