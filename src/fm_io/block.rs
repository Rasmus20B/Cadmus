use crate::fm_format::chunk::{Chunk, ChunkType, BlockErr};
use crate::util::encoding_util::{get_int, put_int};

#[derive(Clone, Debug, Default)]
pub struct Block {
    pub offset: usize,
    pub index: u32,
    pub deleted: bool,
    pub level: u32,
    pub previous: u32,
    pub next: u32,
    pub block_type: u8,
    pub chunks: Vec<ChunkType>,
    pub size: u16,
}

impl Block {
    pub fn header_from_bytes(buffer: &[u8]) -> Self {
        Self {
            offset: 0,
            index: 0,
            deleted: buffer[0] != 0,
            level: buffer[1] as u32 & 0x00FFFFFF,
            previous: get_int(&buffer[4..8]) as u32,
            next: get_int(&buffer[8..12]) as u32,
            block_type: buffer[13],
            chunks: vec![],
            size: 0,
        }
    }

    pub fn new(buffer: &[u8]) -> Self {
        let mut res = Self::header_from_bytes(&buffer);
        res.read_chunks(&buffer).expect("Unable to read chunks.");
        res
    }

    pub fn new_with_index(buffer: &[u8], index: u32) -> Self {
        let mut res = Self::header_from_bytes(&buffer);
        res.index = index;
        res.read_chunks(&buffer).expect("Unable to read chunks.");
        res
    }

    pub fn read_chunks(&mut self, buffer: &[u8]) -> Result<(), String> {
        let mut offset = 20;
        let mut path = vec![];
        let mut instructions_ = vec![];
        let mut size_ = 0;
        while offset < Block::CAPACITY {
            let instruction_res = Chunk::from_bytes(&buffer, &mut offset, &mut path);
            if instruction_res.is_ok() {println!("instructionres: {}", instruction_res.clone().unwrap().chunk_to_string(&buffer));}
            if instruction_res.is_err() {
                let error = instruction_res.clone().unwrap_err();
                match error {
                    BlockErr::EndChunk => {
                        break;
                    },
                    BlockErr::UnrecognizedOpcode(op) => {
                        return Err(format!("Unable to read instruction {:x}. {:?}", op, error));
                    }
                    _ => return Err(format!("{:?}, Chunk Sample: {:x?}", error, &buffer[0..30]))
                }
            }

            let instruction_bind = instruction_res.unwrap();
            size_ += instruction_bind.size() as u16;
            instructions_.push(ChunkType::Unchanged(instruction_bind));
        }

        self.chunks = instructions_;
        
        return Ok(());
    }

    pub fn from_bytes(offset_: usize, index_: Option<u32>, buffer: &[u8]) -> Self {
        let mut offset = 20;
        let mut path = vec![];
        let mut instructions_ = vec![];
        let mut size_ = 0;
        while offset < Block::CAPACITY {
            let instruction_res = Chunk::from_bytes(&buffer, &mut offset, &mut path);
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
            instructions_.push(ChunkType::Unchanged(instruction_bind));
        }

        Self {
            offset: offset_,
            index: index_.unwrap_or(0),
            deleted: buffer[0] != 0,
            level: buffer[1] as u32 & 0x00FFFFFF,
            previous: get_int(&buffer[4..8]) as u32,
            next: get_int(&buffer[8..12]) as u32,
            block_type: buffer[13],
            chunks: instructions_,
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
        buffer[13] = self.block_type;
        buffer
    }
}
