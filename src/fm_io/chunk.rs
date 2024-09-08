use crate::fm_format::instruction::{Instruction, InstructionErr};
use crate::util::encoding_util::get_int;

#[derive(Clone, Debug, Default)]
pub struct Chunk {
    pub offset: usize,
    pub index: u32,
    pub deleted: bool,
    pub level: u32,
    pub previous: u32,
    pub next: u32,
    pub instructions: Vec<Instruction>,
}

impl Chunk {
    pub fn from_buffer(offset_: usize, index_: u32, buffer: &[u8]) -> Self {
        let mut offset = 20;
        let mut path = vec![];
        let mut instructions_ = vec![];
        while offset < Chunk::SIZE {
            let instruction_res = Instruction::from_bytes(&buffer, &mut offset, &mut path);
            if instruction_res.is_err() {
                let error = instruction_res.unwrap_err();
                match error {
                    InstructionErr::EndChunk => {
                        continue;
                    },
                    InstructionErr::UnrecognizedOpcode(op) => {
                        panic!("Unable to read instruction {}. {:?}", op, error);
                    }
                    _ => panic!("{:?}, Chunk Sample: {:x?}", error, &buffer[0..30])
                }
            }

            instructions_.push(instruction_res.unwrap());
        }

        Self {
            offset: offset_,
            index: index_,
            deleted: buffer[0] != 0,
            level: buffer[1] as u32 & 0x00FFFFFF,
            previous: get_int(&buffer[4..8]) as u32,
            next: get_int(&buffer[8..12]) as u32,
            instructions: instructions_,
        }
    }
    pub const SIZE: usize = 4096;
}
