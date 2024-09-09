use core::fmt;
use std::fmt::Formatter;
use crate::{fm_io::{chunk::Chunk, storage::BlockStorage}, util::encoding_util::{get_int, get_path_int}};

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType {
    DataSimple = 0,
    RefSimple = 1,
    RefLong = 2,
    DataSegment = 3,
    PathPush = 4,
    PathPop = 5,
    Noop = 6,
}

#[derive(Debug, Clone, Copy)]
pub enum BlockErr {
    EndChunk,
    UnrecognizedOpcode(u8),
    DataExceedsSectorSize,
    MalformedInstruction,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub offset: u16,
    pub ctype: InstructionType,
    pub opcode: u16,
    pub data: Option<BlockStorage>,
    pub ref_data: Option<BlockStorage>,
    pub path: Vec::<String>,
    pub ref_simple: Option<u16>,
    pub segment_idx: Option<u8>,
}

impl Block {
    pub fn new(
        offset: u16,
        ctype: InstructionType,
           code: u16,
           data: Option<BlockStorage>,
           ref_data: Option<BlockStorage>,
           path: Vec::<String>,
           segment_idx: Option<u8>,
           ref_simple: Option<u16>,
        ) -> Self {
        Self {
            offset,
            ctype,
            opcode: code,
            data,
            ref_data,
            path,
            segment_idx,
            ref_simple,
        }
    }

    pub fn size(&self) -> usize {
        let mut accumulator: usize = 1; // 1 for opcode
        if self.data.is_some() {
            accumulator += self.data.unwrap().length as usize;
        }
        if self.ref_data.is_some() {
            accumulator += self.ref_data.unwrap().length as usize;
        }
        if self.ref_simple.is_some() {
            accumulator += 1;
        }
        if self.segment_idx.is_some() {
            accumulator += 1;
        }
        accumulator
    }

    pub fn get_simple_data(&self, bytes: &[u8]) -> Result<Vec<u8>, BlockErr> {
        let data_uw = self.data.unwrap();
        let range = data_uw.offset as usize..data_uw.offset as usize +data_uw.length as usize;
        Ok(bytes[range].to_vec())
    }

    pub fn to_bytes(&self, chunk_bytes: &Vec<u8>) -> Result<Vec<u8>, BlockErr> {
        let mut buffer: Vec<u8> = vec![];
        buffer.push(self.opcode as u8);
        if self.ref_simple.is_some() {
            buffer.push(self.ref_simple.unwrap() as u8);
        }
        if self.segment_idx.is_some() {
            buffer.push(self.segment_idx.unwrap());
        }
        if self.ref_data.is_some() {
            buffer.extend(self.ref_data.unwrap().lookup_from_buffer(&chunk_bytes).expect("Unable to read offset from buffer."));
        }
        if self.data.is_some() {
            buffer.extend(self.data.unwrap().lookup_from_buffer(&chunk_bytes).expect("Unable to read offset from buffer."));
        }
        Ok(buffer)
    }

    pub fn from_bytes(code: &[u8], offset: &mut usize, path: &mut Vec<String>) -> Result<Block, BlockErr> {
        let mut chunk_code = code[*offset];
        let mut ctype = InstructionType::Noop;
        let mut data: Option<BlockStorage> = None;
        let mut ref_data: Option<BlockStorage> = None;
        let mut segidx: Option<u8> = None;
        let mut ref_simple: Option<u16> = None;
        let mut delayed = 0;
        let saved_offset = offset.clone();
        
        if (chunk_code & 0xC0) == 0xC0 {
            chunk_code &= 0x3F;
            delayed += 1;
        }

        // println!("Chunk: {:x}", chunk_code);

        if chunk_code == 0x00 {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            if *offset >= Chunk::CAPACITY || code[*offset] == 0x0 {
                return Err(BlockErr::EndChunk);
            }
            data = Some(BlockStorage{ offset: *offset as u32, length: 1});
            *offset += 1;
        } else if chunk_code <= 0x05 {
            ctype = InstructionType::RefSimple;
            *offset += 1;
            if *offset >= Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            ref_simple = Some(code[*offset] as u16);
            *offset += 1;
            let len = (chunk_code == 0x01) as usize + (2 * (chunk_code - 0x01) as usize);
            data = Some(BlockStorage{ offset: *offset as u32, length: len as u16});
            *offset += len;
        } else if chunk_code == 0x06 {
            ctype = InstructionType::RefSimple;
            *offset += 1;
            if *offset + 2 > Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            ref_simple = Some(code[*offset] as u16);
            *offset += 1;
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u32, length: len as u16});
            *offset += len;
        } else if chunk_code == 0x07 {
            ctype = InstructionType::DataSegment;
            *offset += 1;
            if *offset +3 > Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            segidx = Some(code[*offset]);
            *offset += 1;
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            data = Some(BlockStorage{ offset: *offset as u32, length: len as u16});
            *offset += len;
        } else if chunk_code == 0x08 {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u32, length: 2 });
            *offset += 2;
        } else if chunk_code == 0x0E && code[21] == 0xFF {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u32, length: 6 });
            *offset += 6;
        } else if chunk_code <= 0x0D {
            ctype = InstructionType::RefSimple;
            *offset += 1;
            if *offset + 2 > Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            ref_simple = Some(get_path_int(&code[*offset..*offset+2]) as u16);
            *offset += 2;
            let len = (chunk_code == 0x09) as usize + (2 *(chunk_code - 0x09) as usize);
            data = Some(BlockStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x0E {
            ctype = InstructionType::RefSimple;
            *offset += 1;
            if *offset + 3 > Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            ref_simple = Some(get_path_int(&code[*offset..*offset+2]) as u16);
            *offset += 2;
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x0F && (code[21] & 0x80 > 0 || (code[*offset+1] & 0x80) > 0) {
            ctype = InstructionType::DataSegment;
            *offset += 2;
            if *offset + 3 > Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            segidx = Some(code[*offset]);
            *offset += 1;
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            data = Some(BlockStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x10 {
                ctype = InstructionType::DataSimple;
                *offset += 1;
                data = Some(BlockStorage{ offset: *offset as u32, length: 3 });
                *offset += 3;
        } else if chunk_code >= 0x11 && chunk_code <= 0x15 {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            let len = 3 + (chunk_code == 0x11) as usize + (2 * (chunk_code as usize - 0x11));
            data = Some(BlockStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x16 {
            ctype = InstructionType::RefLong;
            *offset += 1;
            ref_data = Some(BlockStorage { offset: *offset as u32, length: 3 });
            *offset += 3;
            if *offset >= Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x17 {
            ctype = InstructionType::RefLong;
            *offset += 1;
            ref_data = Some(BlockStorage { offset: *offset as u32, length: 3 });
            *offset += 3;
            if *offset + 2 > Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            data = Some(BlockStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x1B && code[21] == 0 {
            ctype = InstructionType::RefSimple;
            *offset += 2;
            ref_simple = Some(code[*offset] as u16);
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u32, length: 4 });
            *offset += 4;
        } else if chunk_code >= 0x19 && chunk_code <= 0x1D {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            if *offset > Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            let len = code[*offset];
            *offset += 1;
            data = Some(BlockStorage { offset: *offset as u32, length: len as u16 });
            *offset += len as usize + (chunk_code == 0x19) as usize + 2*(chunk_code-0x19) as usize;
        } else if chunk_code == 0x1E {
            ctype = InstructionType::RefLong;
            *offset += 1;
            if *offset >= Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            let ref_len = code[*offset] as usize;
            *offset += 1;
            ref_data = Some(BlockStorage { offset: *offset as u32, length: ref_len as u16 });
            *offset += ref_len;

            if *offset >= Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x1F {
            ctype = InstructionType::RefLong;
            *offset += 1;
            if *offset >= Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            let ref_len = code[*offset] as usize;
            *offset += 1;
            ref_data = Some(BlockStorage { offset: *offset as u32, length: ref_len as u16 });
            *offset += ref_len;
            if *offset + 2 > Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            data = Some(BlockStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x20 {
            ctype = InstructionType::PathPush;
            *offset += 1;
            if *offset >= Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            if code[*offset] == 0xFE {
                *offset += 1;
                data = Some(BlockStorage{ offset: *offset as u32, length: 8 });
            } else {
                data = Some(BlockStorage{ offset: *offset as u32, length: 1 });
            }
            let idx = get_path_int(&code[*offset..*offset+1]);
            *offset += data.unwrap().length as usize;
            path.push(idx.to_string());
        } else if chunk_code == 0x23 {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            if *offset >= Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x28 {
            ctype = InstructionType::PathPush;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u32, length: 2 });
            let idx = get_path_int(&code[*offset..*offset+2]);
            *offset += 2;
            path.push(idx.to_string());
        } else if chunk_code == 0x30 {
            ctype = InstructionType::PathPush;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u32, length: 3 });
            let dir = get_path_int(&code[*offset..*offset+3]).to_string();
            path.push(dir.to_string());
            *offset += 3;
        } else if chunk_code == 0x38 {
            ctype = InstructionType::PathPush;
            *offset += 1;
            if *offset >= Chunk::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u32, length: len as u16 });
            path.push(get_path_int(&code[*offset..*offset+len]).to_string());
            *offset += len;
        } else if chunk_code == 0x3d || chunk_code == 0x40 {
            ctype = InstructionType::PathPop;
            *offset += 1;
            path.pop();
        } else if chunk_code == 0x80 {
            ctype = InstructionType::Noop;
            *offset += 1;
        } else {
            println!("{:?} :: {chunk_code:x}", path);
            return Err(BlockErr::UnrecognizedOpcode(chunk_code));
        };

        while delayed > 0 {
            path.pop();
            delayed -= 1;
        }
        return Ok(Block::new(
            saved_offset as u16,
            ctype,
            chunk_code.into(),
            data,
            ref_data,
            path.clone(),
            segidx,
            ref_simple));
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.ctype {
        InstructionType::DataSegment => {
            write!(f, "path:{:?}::segment:{:?}::data:{:?}::size:{:?}::ins:{:x}", 
                self.path,
                 self.segment_idx.unwrap(),
                 self.data.unwrap(),
                 self.data.unwrap().length,
                 self.opcode)
        },
        InstructionType::RefSimple => {
            write!(f, "path:{:?}::reference:{:?}::ref_data:{:?}::size:{}::ins:{:x}", 
                 self.path,
                 self.ref_simple.unwrap(),
                 self.data.unwrap(),
                 self.data.unwrap().length,
                 self.opcode)
        }
        InstructionType::DataSimple => {
            write!(f, "path:{:?}::reference:na::ref_data:{:?}::size:{}::ins:{:x}", 
                 self.path,
                 self.data.unwrap(),
                 self.data.unwrap().length,
                 self.opcode)
        }
        InstructionType::RefLong => {
            write!(f, "reference:{:?}::ref_data:{:?}::size:{}::ins:{:x}", 
                 self.ref_data.unwrap(),
                 self.data.unwrap(),
                 self.data.unwrap().length,
                 self.opcode)
        }
        InstructionType::PathPush => {
            write!(f, "path:{:?}::reference:PUSH::ref_data:{:?}::size:{}::ins:{:x}", 
                 self.path,
                 self.data.unwrap(),
                 self.data.unwrap().length,
                 self.opcode)
        }
        InstructionType::PathPop => {
            write!(f, "path:{:?}::reference:POP::ref_data:None::size:{}::ins:{:x}", 
                 self.path,
                 0,
                 self.opcode)
        }
        InstructionType::Noop => {
            write!(f, "path:{:?}::reference:NOOP::ref_data:None::size:{}::ins:{:x}", 
                 self.path,
                 0,
                 self.opcode)
        }
    }
    }
}
