use crate::fm_io::{block::Block, storage::BlockStorage};


pub struct DataStaging {
    pub buffer: Vec<u8>,
    offset: u16,
}

impl DataStaging {
    pub fn new() -> Self {
        Self {
            buffer: vec![0u8; 4096],
            offset: 0,
        }
    }
    pub fn store(&mut self, value: Vec<u8>) -> BlockStorage {

        /* store each byte when we reach an empty area in buffer. */
        let ret = BlockStorage { offset: self.offset, length: value.len() as u16 };
        self.offset += value.len() as u16;
        self.buffer.splice(self.offset as usize - value.len()..self.offset as usize, value);
        /* return the offset and the length of the data in a BlockStorage struct. */
        ret
    }

    pub fn load(&self, location: BlockStorage) -> Vec<u8> {
        self.buffer[location.offset as usize..(location.offset+location.length) as usize].to_vec()
    }
}
