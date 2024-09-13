use core::fmt;
use std::{fmt::Formatter, ops::RangeBounds};
use crate::{fm_io::{block::Block, storage::BlockStorage}, staging_buffer::DataStaging, util::encoding_util::{get_int, get_path_int}};

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

#[derive(Debug, Clone)]
pub enum ChunkType {
    Modification(Chunk),
    Unchanged(Chunk),
}
impl From<ChunkType> for Chunk {
    fn from(chunk_wrapper: ChunkType) -> Chunk {
        match chunk_wrapper {
            ChunkType::Modification(chunk) | ChunkType::Unchanged(chunk) => chunk,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub offset: u16,
    pub ctype: InstructionType,
    pub opcode: u16,
    pub data: Option<BlockStorage>,
    pub ref_data: Option<BlockStorage>,
    pub path: Vec::<String>,
    pub ref_simple: Option<u16>,
    pub segment_idx: Option<u8>,
}

impl Chunk {
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

    pub fn to_bytes(&self, chunk_bytes: &DataStaging) -> Result<Vec<u8>, BlockErr> {
        let mut buffer: Vec<u8> = vec![];
        if (self.opcode & 0xFF00) == 0 {
            buffer.push(self.opcode as u8);
        } else {
            println!("here.");
            buffer.push((self.opcode &0x00FF) as u8);
            buffer.push((self.opcode << 8) as u8);
        }
        if self.ref_simple.is_some() {
            let ref_uw = self.ref_simple.unwrap();
            if ref_uw > u8::max_value().into() {
                let bytes = self.ref_simple.unwrap().to_be_bytes();
                buffer.extend(bytes);
            } else {
                buffer.push(self.ref_simple.unwrap() as u8);
            }
        }
        if self.segment_idx.is_some() {
            buffer.push(self.segment_idx.unwrap());
        }
        if self.ref_data.is_some() {
            buffer.push(self.ref_data.unwrap().length as u8);
            buffer.extend(chunk_bytes.load(self.ref_data.unwrap()));
        }
        if self.data.is_some() {
            if self.ctype != InstructionType::PathPush &&
                !(0x00..0x05).contains(&self.opcode) &&
                !(0x11..0x15).contains(&self.opcode) {
                if self.opcode == 0x1b {
                    buffer.push((self.data.unwrap().length - 4) as u8);
                } else {
                    buffer.push(self.data.unwrap().length as u8);
                }
            } 

            buffer.extend(chunk_bytes.load(self.data.unwrap()));
        }
        Ok(buffer)
    }

    pub fn from_bytes(code: &[u8], offset: &mut usize, path: &mut Vec<String>) -> Result<Chunk, BlockErr> {
        let mut chunk_code = code[*offset];
        let mut ctype = InstructionType::Noop;
        let mut data: Option<BlockStorage> = None;
        let mut ref_data: Option<BlockStorage> = None;
        let mut segidx: Option<u8> = None;
        let mut ref_simple: Option<u16> = None;
        let mut delayed = 0;
        let saved_offset = offset.clone();
        let mut saved_chunk_code = chunk_code as u16;
        
        if (chunk_code & 0xC0) == 0xC0 {
            chunk_code &= 0x3F;
            delayed += 1;
        }

        // println!("Chunk: {:x}", chunk_code);

        if chunk_code == 0x00 {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            if *offset >= Block::CAPACITY || code[*offset] == 0x0 {
                return Err(BlockErr::EndChunk);
            }
            data = Some(BlockStorage{ offset: *offset as u16, length: 1});
            *offset += 1;
        } else if chunk_code <= 0x05 {
            ctype = InstructionType::RefSimple;
            *offset += 1;
            if *offset >= Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            ref_simple = Some(code[*offset] as u16);
            *offset += 1;
            let len = (chunk_code == 0x01) as usize + (2 * (chunk_code - 0x01) as usize);
            data = Some(BlockStorage{ offset: *offset as u16, length: len as u16});
            *offset += len;
        } else if chunk_code == 0x06 {
            ctype = InstructionType::RefSimple;
            *offset += 1;
            if *offset + 2 > Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            ref_simple = Some(code[*offset] as u16);
            *offset += 1;
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u16, length: len as u16});
            *offset += len;
        } else if chunk_code == 0x07 {
            ctype = InstructionType::DataSegment;
            *offset += 1;
            if *offset +3 > Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            segidx = Some(code[*offset]);
            *offset += 1;
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            data = Some(BlockStorage{ offset: *offset as u16, length: len as u16});
            *offset += len;
        } else if chunk_code == 0x08 {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u16, length: 2 });
            *offset += 2;
        } else if chunk_code == 0x0E && code[21] == 0xFF {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u16, length: 6 });
            *offset += 6;
        } else if chunk_code <= 0x0D {
            ctype = InstructionType::RefSimple;
            *offset += 1;
            if *offset + 2 > Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            ref_simple = Some(get_path_int(&code[*offset..*offset+2]) as u16);
            *offset += 2;
            let len = (chunk_code == 0x09) as usize + (2 *(chunk_code - 0x09) as usize);
            data = Some(BlockStorage{ offset: *offset as u16, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x0E {
            ctype = InstructionType::RefSimple;
            *offset += 1;
            if *offset + 3 > Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            ref_simple = Some(u16::from_be_bytes(code[*offset..*offset+2].try_into().expect("CN")));
            *offset += 2;
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u16, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x0F && (code[21] & 0x80 > 0 || (code[*offset+1] & 0x80) > 0) {
            ctype = InstructionType::DataSegment;
            if (code[*offset+1] & 0x80) > 0 {
                saved_chunk_code = (saved_chunk_code << 8) + code[*offset+1] as u16;
            }
            *offset += 2;
            if *offset + 3 > Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            segidx = Some(code[*offset]);
            *offset += 1;
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            data = Some(BlockStorage{ offset: *offset as u16, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x10 {
                ctype = InstructionType::DataSimple;
                *offset += 1;
                data = Some(BlockStorage{ offset: *offset as u16, length: 3 });
                *offset += 3;
        } else if chunk_code >= 0x11 && chunk_code <= 0x15 {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            let len = 3 + ((chunk_code == 0x11) as usize) + (2 * (chunk_code as usize - 0x11));
            data = Some(BlockStorage{ offset: *offset as u16, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x16 {
            ctype = InstructionType::RefLong;
            *offset += 1;
            ref_data = Some(BlockStorage { offset: *offset as u16, length: 3 });
            *offset += 3;
            if *offset >= Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u16, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x17 {
            ctype = InstructionType::RefLong;
            *offset += 1;
            ref_data = Some(BlockStorage { offset: *offset as u16, length: 3 });
            *offset += 3;
            if *offset + 2 > Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            data = Some(BlockStorage{ offset: *offset as u16, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x1B && code[*offset+1] == 0 {
            saved_chunk_code = (saved_chunk_code << 8) + code[*offset+1] as u16;
            ctype = InstructionType::RefSimple;
            *offset += 2;
            ref_simple = Some(code[*offset] as u16);
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u16, length: 4 });
            *offset += 4;
        } else if chunk_code >= 0x19 && chunk_code <= 0x1D {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            if *offset > Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            let len = code[*offset] + (((chunk_code == 0x19) as usize) + (2*(chunk_code-0x19) as usize)) as u8;
            *offset += 1;
            data = Some(BlockStorage { offset: *offset as u16, length: len as u16 });
            *offset += len as usize;
        } else if chunk_code == 0x1E {
            ctype = InstructionType::RefLong;
            *offset += 1;
            if *offset >= Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            let ref_len = code[*offset] as usize;
            *offset += 1;
            ref_data = Some(BlockStorage { offset: *offset as u16, length: ref_len as u16 });
            *offset += ref_len;

            if *offset >= Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u16, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x1F {
            ctype = InstructionType::RefLong;
            *offset += 1;
            if *offset >= Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            let ref_len = code[*offset] as usize;
            *offset += 1;
            ref_data = Some(BlockStorage { offset: *offset as u16, length: ref_len as u16 });
            *offset += ref_len;
            if *offset + 2 > Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            data = Some(BlockStorage{ offset: *offset as u16, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x20 {
            ctype = InstructionType::PathPush;
            *offset += 1;
            if *offset >= Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            if code[*offset] == 0xFE {
                *offset += 1;
                data = Some(BlockStorage{ offset: *offset as u16, length: 8 });
            } else {
                data = Some(BlockStorage{ offset: *offset as u16, length: 1 });
            }
            let idx = get_path_int(&code[*offset..*offset+1]);
            *offset += data.unwrap().length as usize;
            path.push(idx.to_string());
        } else if chunk_code == 0x23 {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            if *offset >= Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize)
            }
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u16, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x28 {
            ctype = InstructionType::PathPush;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u16, length: 2 });
            let idx = get_path_int(&code[*offset..*offset+2]);
            *offset += 2;
            path.push(idx.to_string());
        } else if chunk_code == 0x30 {
            ctype = InstructionType::PathPush;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u16, length: 3 });
            let dir = get_path_int(&code[*offset..*offset+3]).to_string();
            path.push(dir.to_string());
            *offset += 3;
        } else if chunk_code == 0x38 {
            ctype = InstructionType::PathPush;
            *offset += 1;
            if *offset >= Block::CAPACITY {
                return Err(BlockErr::DataExceedsSectorSize);
            }
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(BlockStorage{ offset: *offset as u16, length: len as u16 });
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

        let chunk = Chunk::new(
                        saved_offset as u16,
                        ctype,
                        saved_chunk_code.into(),
                        data,
                        ref_data,
                        path.clone(),
                        segidx,
                        ref_simple);
        if delayed > 0 {
            path.pop();
        }
        return Ok(chunk)
    }
}

impl fmt::Display for Chunk {
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
            write!(f, "path:{:?}::reference:na::simple_data:{:?}::size:{}::ins:{:x}", 
                 self.path,
                 self.data.unwrap(),
                 self.data.unwrap().length,
                 self.opcode)
        }
        InstructionType::RefLong => {
            write!(f, "path:{:?}::reference:{:?}::ref_data:{:?}::size:{}::ins:{:x}", 
                 self.path,
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
