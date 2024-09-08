use core::fmt;
use std::fmt::Formatter;
use crate::{fm_io::{chunk::Chunk, storage::ChunkStorage}, util::encoding_util::{get_int, get_path_int}};

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

#[derive(Debug)]
pub enum InstructionErr {
    EndChunk,
    UnrecognizedOpcode(u8),
    DataExceedsSectorSize,
}

#[derive(Clone, Debug)]
pub struct Instruction {
    pub ctype: InstructionType,
    pub opcode: u16,
    pub data: Option<ChunkStorage>,
    pub ref_data: Option<ChunkStorage>,
    pub path: Vec::<String>,
    pub ref_simple: Option<u16>,
    pub segment_idx: Option<u8>,
}

impl Instruction {
    pub fn new(ctype: InstructionType,
           code: u16,
           data: Option<ChunkStorage>,
           ref_data: Option<ChunkStorage>,
           path: Vec::<String>,
           segment_idx: Option<u8>,
           ref_simple: Option<u16>,
        ) -> Self {
        Self {
            ctype,
            opcode: code,
            data,
            ref_data,
            path,
            segment_idx,
            ref_simple,
        }
    }

    pub fn from_bytes(code: &[u8], offset: &mut usize, path: &mut Vec<String>) -> Result<Instruction, InstructionErr> {
        let mut chunk_code = code[*offset];
        let mut ctype = InstructionType::Noop;
        let mut data: Option<ChunkStorage> = None;
        let mut ref_data: Option<ChunkStorage> = None;
        let mut segidx: Option<u8> = None;
        let mut ref_simple: Option<u16> = None;
        let mut delayed = 0;
        
        if (chunk_code & 0xC0) == 0xC0 {
            chunk_code &= 0x3F;
            delayed += 1;
        }

        if chunk_code == 0x00 {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            if *offset >= Chunk::SIZE || code[*offset] == 0x0 {
                return Err(InstructionErr::EndChunk);
            }
            data = Some(ChunkStorage{ offset: *offset as u32, length: 1});
            *offset += 1;
        } else if chunk_code <= 0x05 {
            ctype = InstructionType::RefSimple;
            *offset += 1;
            if *offset >= Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize);
            }
            ref_simple = Some(code[*offset] as u16);
            *offset += 1;
            let len = (chunk_code == 0x01) as usize + (2 * (chunk_code - 0x01) as usize);
            data = Some(ChunkStorage{ offset: *offset as u32, length: len as u16});
            *offset += len;
        } else if chunk_code == 0x06 {
            ctype = InstructionType::RefSimple;
            *offset += 1;
            if *offset + 2 > Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize);
            }
            ref_simple = Some(code[*offset] as u16);
            *offset += 1;
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(ChunkStorage{ offset: *offset as u32, length: len as u16});
            *offset += len;
        } else if chunk_code == 0x07 {
            ctype = InstructionType::DataSegment;
            *offset += 1;
            if *offset +3 > Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize);
            }
            segidx = Some(code[*offset]);
            *offset += 1;
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            data = Some(ChunkStorage{ offset: *offset as u32, length: len as u16});
            *offset += len;
        } else if chunk_code == 0x08 {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            data = Some(ChunkStorage{ offset: *offset as u32, length: 2 });
            *offset += 2;
        } else if chunk_code == 0x0E && code[21] == 0xFF {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            data = Some(ChunkStorage{ offset: *offset as u32, length: 6 });
            *offset += 6;
        } else if chunk_code <= 0x0D {
            ctype = InstructionType::RefSimple;
            *offset += 1;
            if *offset + 2 > Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize);
            }
            ref_simple = Some(get_path_int(&code[*offset..*offset+2]) as u16);
            *offset += 2;
            let len = (chunk_code == 0x09) as usize + (2 *(chunk_code - 0x09) as usize);
            data = Some(ChunkStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x0E {
            ctype = InstructionType::RefSimple;
            *offset += 1;
            if *offset + 3 > Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize);
            }
            ref_simple = Some(get_path_int(&code[*offset..*offset+2]) as u16);
            *offset += 2;
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(ChunkStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x0F && (code[21] & 0x80 > 0 || (code[*offset+1] & 0x80) > 0) {
            ctype = InstructionType::DataSegment;
            *offset += 2;
            if *offset + 3 > Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize)
            }
            segidx = Some(code[*offset]);
            *offset += 1;
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            data = Some(ChunkStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x10 {
                ctype = InstructionType::DataSimple;
                *offset += 1;
                data = Some(ChunkStorage{ offset: *offset as u32, length: 3 });
                *offset += 3;
        } else if chunk_code >= 0x11 && chunk_code <= 0x15 {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            let len = 3 + (chunk_code == 0x11) as usize + (2 * (chunk_code as usize - 0x11));
            data = Some(ChunkStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x16 {
            ctype = InstructionType::RefLong;
            *offset += 1;
            ref_data = Some(ChunkStorage { offset: *offset as u32, length: 3 });
            *offset += 3;
            if *offset >= Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize);
            }
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(ChunkStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x17 {
            ctype = InstructionType::RefLong;
            *offset += 1;
            ref_data = Some(ChunkStorage { offset: *offset as u32, length: 3 });
            *offset += 3;
            if *offset + 2 > Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize);
            }
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            data = Some(ChunkStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x1B && code[21] == 0 {
            ctype = InstructionType::RefSimple;
            *offset += 2;
            ref_simple = Some(code[*offset] as u16);
            *offset += 1;
            data = Some(ChunkStorage{ offset: *offset as u32, length: 4 });
            *offset += 4;
        } else if chunk_code >= 0x19 && chunk_code <= 0x1D {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            if *offset > Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize)
            }
            let len = code[*offset];
            *offset += 1;
            data = Some(ChunkStorage { offset: *offset as u32, length: len as u16 });
            *offset += len as usize + (chunk_code == 0x19) as usize + 2*(chunk_code-0x19) as usize;
        } else if chunk_code == 0x1E {
            ctype = InstructionType::RefLong;
            *offset += 1;
            if *offset >= Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize)
            }
            let ref_len = code[*offset] as usize;
            *offset += 1;
            ref_data = Some(ChunkStorage { offset: *offset as u32, length: ref_len as u16 });
            *offset += ref_len;

            if *offset >= Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize)
            }
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(ChunkStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x1F {
            ctype = InstructionType::RefLong;
            *offset += 1;
            if *offset >= Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize)
            }
            let ref_len = code[*offset] as usize;
            *offset += 1;
            ref_data = Some(ChunkStorage { offset: *offset as u32, length: ref_len as u16 });
            *offset += ref_len;
            if *offset + 2 > Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize)
            }
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            data = Some(ChunkStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x20 {
            ctype = InstructionType::PathPush;
            *offset += 1;
            if *offset >= Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize)
            }
            if code[*offset] == 0xFE {
                *offset += 1;
                data = Some(ChunkStorage{ offset: *offset as u32, length: 8 });
            } else {
                data = Some(ChunkStorage{ offset: *offset as u32, length: 1 });
            }
            let idx = get_path_int(&code[*offset..*offset+1]);
            *offset += data.unwrap().length as usize;
            path.push(idx.to_string());
        } else if chunk_code == 0x23 {
            ctype = InstructionType::DataSimple;
            *offset += 1;
            if *offset >= Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize)
            }
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(ChunkStorage{ offset: *offset as u32, length: len as u16 });
            *offset += len;
        } else if chunk_code == 0x28 {
            ctype = InstructionType::PathPush;
            *offset += 1;
            data = Some(ChunkStorage{ offset: *offset as u32, length: 2 });
            let idx = get_path_int(&code[*offset..*offset+2]);
            *offset += 2;
            path.push(idx.to_string());
        } else if chunk_code == 0x30 {
            ctype = InstructionType::PathPush;
            *offset += 1;
            data = Some(ChunkStorage{ offset: *offset as u32, length: 3 });
            let dir = get_path_int(&code[*offset..*offset+3]).to_string();
            path.push(dir.to_string());
            *offset += 3;
        } else if chunk_code == 0x38 {
            ctype = InstructionType::PathPush;
            *offset += 1;
            if *offset >= Chunk::SIZE {
                return Err(InstructionErr::DataExceedsSectorSize);
            }
            let len = code[*offset] as usize;
            *offset += 1;
            data = Some(ChunkStorage{ offset: *offset as u32, length: len as u16 });
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
            return Err(InstructionErr::UnrecognizedOpcode(chunk_code));
        };

        while delayed > 0 {
            path.pop();
            delayed -= 1;
        }
        return Ok(Instruction::new(
            ctype,
            chunk_code.into(),
            data,
            ref_data,
            path.clone(),
            segidx,
            ref_simple));
    }
}

impl fmt::Display for Instruction {
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
