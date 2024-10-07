use core::fmt;
use serde_json;
use serde::{self, Deserialize, Serialize};
use std::{fmt::Formatter, ops::RangeBounds};
use crate::staging_buffer::DataStaging;
use crate::util::encoding_util::{get_int, get_path_int, put_int, put_path_int};

use super::{block::Block, btree::HBAMFile, path::HBAMPath, block_location::BlockLocation, storage::BlockStorage};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ChunkType {
    Modification(Chunk),
    Unchanged(Chunk),
}

impl ChunkType {
    pub const fn chunk(&self) -> &Chunk {
        match self {
            ChunkType::Modification(chunk) | ChunkType::Unchanged(chunk) => chunk
        }
    }

    pub fn chunk_mut(&mut self) -> &mut Chunk {
        match self {
            ChunkType::Modification(chunk) | ChunkType::Unchanged(chunk) => chunk
        }
    }
}

impl From<ChunkType> for Chunk {
    fn from(chunk_wrapper: ChunkType) -> Chunk {
        match chunk_wrapper {
            ChunkType::Modification(chunk) | ChunkType::Unchanged(chunk) => chunk,
        }
    }
}

impl From<&ChunkType> for Chunk {
    fn from(chunk_wrapper: &ChunkType) -> Chunk {
        match chunk_wrapper {
            ChunkType::Modification(chunk) | ChunkType::Unchanged(chunk) => chunk.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chunk {
    pub offset: u16,
    pub ctype: InstructionType,
    pub opcode: u16,
    pub data: Option<BlockStorage>,
    pub ref_data: Option<BlockStorage>,
    pub path: HBAMPath,
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
           path: HBAMPath,
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

        let mut new_opcode = self.opcode;

        if (new_opcode & 0xFF00) == 0 {
            if (0x01..=0x05).contains(&new_opcode) {
                let len = self.data.unwrap().length;
                new_opcode = match len {
                    1 => 1,
                    _ => (len / 2) + 1,
                };

                if new_opcode >= 0x06 {
                    new_opcode = 0x06;
                }

                // println!("FOUND ANEW OPCODE: {}", new_opcode);

                buffer.push(new_opcode as u8);
            } else if (0x11..=0x15).contains(&new_opcode) {
                let len = self.data.unwrap().length;
                new_opcode = match len {
                    1 => 3,
                    _ => 3 + (len / 2) + 1,
                };
                buffer.push(new_opcode as u8);
            } else {
                buffer.push(new_opcode as u8);
            }
        } else {
            buffer.push((new_opcode &0x00FF) as u8);
            buffer.push((new_opcode << 8) as u8);
        }

        if self.ctype == InstructionType::PathPop || self.ctype == InstructionType::Noop { 
            return Ok(buffer);
        }

        if self.ref_simple.is_some() {
            if new_opcode == 0xe {
                let n_buf = u16::to_be_bytes(self.ref_simple.unwrap());
                buffer.append(&mut n_buf.to_vec());
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
            if (0x01..=0x05).contains(&new_opcode) ||
                (0x11..=0x15).contains(&new_opcode) {
            } else if new_opcode == 0x1b && self.data.unwrap().length == 4{
                buffer.push(0);
            } else if new_opcode == 0x7 
                || new_opcode == 0x0f
                || new_opcode == 0x17
                || new_opcode == 0x1f {
                let len_buf = u16::to_be_bytes(self.data.unwrap().length);
                buffer.append(&mut len_buf.to_vec());
            } else if new_opcode >= 0x19 && new_opcode <= 0x1D {
                // println!("path: {:?}, len: {}, calc: {}", self.path, self.data.unwrap().length, (((new_opcode == 0x19) as u8) + (2*new_opcode as u8 - 0x19)) as u8);
                // let extra = self.data.unwrap().length as u8 + (((new_opcode == 0x19) as u8) + (2*new_opcode as u8 - 0x19)) as u8;
                // let len = code[*offset] + (((chunk_code == 0x19) as usize) + (2*(chunk_code-0x19) as usize)) as u8;
                // buffer.push(extra as u8);
                buffer.push(self.data.unwrap().length as u8 - 4);
            } else if new_opcode == 0x20 {
            } else if new_opcode == 0x0 {
            } else if new_opcode == 0x28 {
            } else if new_opcode == 0x30 {
            } else if new_opcode == 0x38 {
                buffer.push(self.data.unwrap().length as u8);
            } else {
                // println!("DATA LENGTH: {}", self.data.unwrap().length);
                buffer.push(self.data.unwrap().length as u8);
            }

            if new_opcode == 0x28 {
                let mut idx = self.data.unwrap().lookup_from_buffer(&chunk_bytes.buffer).expect("Unable to lookup from buffer");
                buffer.append(&mut idx);
            } else {
                buffer.extend(chunk_bytes.load(self.data.unwrap()));
            }
        } 

        if new_opcode == 0x30 {
            // println!("len: {}, new opcode: {}",self.data.unwrap().length, new_opcode);
            // println!("Chunk: {}\nbuffer: {:x?}", self, buffer);
        }

        Ok(buffer)
    }

    pub fn from_bytes(code: &[u8], offset: &mut usize, path: &mut Vec<String>) -> Result<Chunk, BlockErr> {
        let mut chunk_code = code[*offset];
        let ctype;
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

        // if chunk_code == 0x1b {
        //     println!("path: {:?}, BUf: {:x?}", path, &code[*offset..*offset+15]);
        // }
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
            ref_simple = Some(get_int(&code[*offset..*offset+2]) as u16);
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
            let dir = get_path_int(&code[*offset+1..*offset+3]).to_string();
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
                        HBAMPath::new(path.to_vec()),
                        segidx,
                        ref_simple);
        if delayed > 0 {
            path.pop();
        }
        Ok(chunk)
    }

    pub fn chunk_to_string(&self, buffer: &[u8]) -> String {
        match self.ctype {
            InstructionType::DataSegment => {
                format!("path:{:?}::segment:{:?}::data:{:?}::size:{:?}::ins:{:x}", 
                    self.path.components,
                     self.segment_idx.unwrap(),
                     self.data.unwrap().lookup_from_buffer(buffer),
                     self.data.unwrap().length,
                     self.opcode)
            },
            InstructionType::RefSimple => {
                format!("path:{:?}::reference:{:?}::ref_data:{:?}::size:{}::ins:{:x}", 
                     self.path.components,
                     self.ref_simple.unwrap(),
                     self.data.unwrap().lookup_from_buffer(buffer),
                     self.data.unwrap().length,
                     self.opcode)
            }
            InstructionType::DataSimple => {
                format!("path:{:?}::reference:na::simple_data:{:?}::size:{}::ins:{:x}", 
                     self.path.components,
                     self.data.unwrap().lookup_from_buffer(buffer),
                     self.data.unwrap().length,
                     self.opcode)
            }
            InstructionType::RefLong => {
                format!("path:{:?}::reference:{:?}::ref_data:{:?}::size:{}::ins:{:x}", 
                     self.path.components,
                     self.ref_data.unwrap().lookup_from_buffer(buffer),
                     self.data.unwrap().lookup_from_buffer(buffer),
                     self.data.unwrap().length,
                     self.opcode)
            }
            InstructionType::PathPush => {
                format!("path:{:?}::reference:PUSH::ref_data:{:?}::size:{}::ins:{:x}", 
                     self.path.components,
                     self.data.unwrap().lookup_from_buffer(buffer),
                     self.data.unwrap().length,
                     self.opcode)
            }
            InstructionType::PathPop => {
                format!("path:{:?}::reference:POP::ref_data:None::size:{}::ins:{:x}", 
                     self.path.components,
                     0,
                     self.opcode)
            }
            InstructionType::Noop => {
                format!("path:{:?}::reference:NOOP::ref_data:None::size:{}::ins:{:x}", 
                     self.path.components,
                     0,
                     self.opcode)
            }
        }
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.ctype {
        InstructionType::DataSegment => {
            write!(f, "path:{:?}::segment:{:?}::data:{:?}::size:{:?}::ins:{:x}", 
                self.path.components,
                 self.segment_idx.unwrap(),
                 self.data.unwrap(),
                 self.data.unwrap().length,
                 self.opcode)
        },
        InstructionType::RefSimple => {
            write!(f, "path:{:?}::reference:{:?}::ref_data:{:?}::size:{}::ins:{:x}", 
                 self.path.components,
                 self.ref_simple.unwrap(),
                 self.data.unwrap(),
                 self.data.unwrap().length,
                 self.opcode)
        }
        InstructionType::DataSimple => {
            write!(f, "path:{:?}::reference:na::simple_data:{:?}::size:{}::ins:{:x}", 
                 self.path.components,
                 self.data.unwrap(),
                 self.data.unwrap().length,
                 self.opcode)
        }
        InstructionType::RefLong => {
            write!(f, "path:{:?}::reference:{:?}::ref_data:{:?}::size:{}::ins:{:x}", 
                 self.path.components,
                 self.ref_data.unwrap(),
                 self.data.unwrap(),
                 self.data.unwrap().length,
                 self.opcode)
        }
        InstructionType::PathPush => {
            write!(f, "path:{:?}::reference:PUSH::ref_data:{:?}::size:{}::ins:{:x}", 
                 self.path.components,
                 self.data.unwrap(),
                 self.data.unwrap().length,
                 self.opcode)
        }
        InstructionType::PathPop => {
            write!(f, "path:{:?}::reference:POP::ref_data:None::size:{}::ins:{:x}", 
                 self.path.components,
                 0,
                 self.opcode)
        }
        InstructionType::Noop => {
            write!(f, "path:{:?}::reference:NOOP::ref_data:None::size:{}::ins:{:x}", 
                 self.path.components,
                 0,
                 self.opcode)
        }
    }
    }
}
