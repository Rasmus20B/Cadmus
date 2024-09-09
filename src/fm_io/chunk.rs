use crate::fm_format::block::{Block, BlockErr};
use crate::util::encoding_util::{get_int, put_int};

#[derive(Clone, Debug, Default)]
pub struct Chunk {
    pub offset: usize,
    pub index: u32,
    pub deleted: bool,
    pub level: u32,
    pub previous: u32,
    pub next: u32,
    pub instructions: Vec<Block>,
    pub size: u16,
}

impl Chunk {
    pub fn from_bytes(offset_: usize, index_: u32, buffer: &[u8]) -> Self {
        let mut offset = 20;
        let mut path = vec![];
        let mut instructions_ = vec![];
        let mut size_ = 0;
        while offset < Chunk::CAPACITY {
            let instruction_res = Block::from_bytes(&buffer, &mut offset, &mut path);
            if instruction_res.is_err() {
                let error = instruction_res.unwrap_err();
                match error {
                    BlockErr::EndChunk => {
                        continue;
                    },
                    BlockErr::UnrecognizedOpcode(op) => {
                        panic!("Unable to read instruction {}. {:?}", op, error);
                    }
                    _ => panic!("{:?}, Chunk Sample: {:x?}", error, &buffer[0..30])
                }
            }

            let instruction_bind = instruction_res.unwrap();
            size_ += instruction_bind.size() as u16;
            instructions_.push(instruction_bind);
        }

        Self {
            offset: offset_,
            index: index_,
            deleted: buffer[0] != 0,
            level: buffer[1] as u32 & 0x00FFFFFF,
            previous: get_int(&buffer[4..8]) as u32,
            next: get_int(&buffer[8..12]) as u32,
            instructions: instructions_,
            size: size_,
        }
    }
    pub const CAPACITY: usize = 4096;
    
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = vec![0u8; 20];
        buffer[0] = self.deleted as u8;
        /* level is 0 so no change necessary from default u8. */
        buffer.splice(4..8, put_int(self.previous as usize));
        buffer.splice(8..12, put_int(self.next as usize));
        buffer
    }
}
