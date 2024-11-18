use core::fmt;
use std::fmt::Formatter;
use serde::{self, Deserialize, Serialize};
use crate::util::encoding_util::{get_int, get_path_int};

use super::page::Page;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ChunkContents<'a> {
    SimpleData { data: &'a [u8] },
    SimpleRef { key: u16, data: &'a [u8] },
    LongRef { key: &'a [u8], data: &'a [u8] },
    Segment { index: u16, data: &'a [u8] },
    Push { key: &'a [u8] },
    Pop,
    Noop, 
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LocalChunkContents {
    SimpleData { data: Vec<u8> },
    SimpleRef { key: u16, data: Vec<u8> },
    LongRef { key: Vec<u8>, data: Vec<u8> },
    Segment { index: u16, data: Vec<u8> },
    Push { key: Vec<u8> },
    Pop,
    Noop, 
}

#[derive(Debug, Clone, Copy)]
pub enum ParseErr {
    EndChunk,
    UnexpectedEndOfPage,
    UnrecognizedOpcode(u8),
    MalformedChunk,
    DataExceedsSectorSize,
}

#[derive(Debug, Clone)]
pub struct LocalChunk {
    pub offset: u16,
    pub opcode: u16,
    pub contents: LocalChunkContents,
    pub delayed: bool,
}

#[derive(Debug, Clone)]
pub struct Chunk<'a> {
    pub offset: u16,
    pub opcode: u16,
    pub contents: ChunkContents<'a>,
    pub delayed: bool,
}

impl<'a> Chunk<'a> {
    pub fn new(
        offset_: u16,
        code_: u16,
        contents_: ChunkContents<'a>,
        delayed_: bool,
        ) -> Chunk<'a> {
        Self {
            offset: offset_,
            opcode: code_,
            contents: contents_,
            delayed: delayed_,
        }
    }

    pub fn copy_to_local(&self) -> LocalChunk {
        LocalChunk {
            offset: self.offset,
            opcode: self.opcode,
            delayed: self.delayed,
            contents: match self.contents {
                ChunkContents::SimpleData { data } => {
                    LocalChunkContents::SimpleData { data: data.to_vec() }
                }
                ChunkContents::SimpleRef { key, data } => {
                    LocalChunkContents::SimpleRef { key: key.clone(), data: data.to_vec() }
                }
                ChunkContents::LongRef { key, data } => {
                    LocalChunkContents::LongRef { key: key.to_vec(), data: data.to_vec() }
                }
                ChunkContents::Pop => {
                    LocalChunkContents::Pop
                },
                ChunkContents::Push { key } => {
                    LocalChunkContents::Push { key: key.to_vec() }
                }
                ChunkContents::Segment { ref index, data } => {
                    LocalChunkContents::Segment { index: *index, data: data.to_vec() }
                }
                ChunkContents::Noop => {
                    LocalChunkContents::Noop
                }
            }
        }
    }

    pub fn from_bytes<'b>(code: &'a [u8], offset: &'b mut usize) -> Result<Chunk<'b>, ParseErr> 
        where 'a: 'b {
        let mut chunk_code = code[*offset];
        let mut delayed = false;
        let saved_offset = *offset as u16;

        let mut saved_chunk_code = chunk_code as u16;
        let mut contents: ChunkContents = ChunkContents::Noop;
        
        if (chunk_code & 0xC0) == 0xC0 {
            chunk_code &= 0x3F;
            delayed = true;
        }

        if chunk_code == 0x00 {
            *offset += 1;
            if *offset as u64 >= Page::SIZE || code[*offset] == 0x0 {
                return Err(ParseErr::EndChunk);
            }
            contents = ChunkContents::SimpleData { data: &code[*offset..=*offset+1] };
            *offset += 1;
        } else if chunk_code <= 0x05 {
            *offset += 1;
            if *offset as u64 >= Page::SIZE {
                return Err(ParseErr::DataExceedsSectorSize);
            }
            let key_ = code[*offset] as u16;
            *offset += 1;
            let len = (chunk_code == 0x01) as usize + (2 * (chunk_code - 0x01) as usize);
            contents = ChunkContents::SimpleRef { 
                key: key_,
                data: &code[*offset..*offset+len],
            };
            *offset += len;
        } else if chunk_code == 0x06 {
            *offset += 1;
            if *offset + 2 > Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize);
            }
            let key_ = code[*offset] as u16;
            *offset += 1;
            let len = code[*offset] as usize;
            *offset += 1;
            contents = ChunkContents::SimpleRef { 
                key: key_,
                data: &code[*offset..*offset+len], 
            };
            *offset += len;
        } else if chunk_code == 0x07 {
            *offset += 1;
            if *offset +3 > Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize);
            }
            let segidx = code[*offset];
            *offset += 1;
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            contents = ChunkContents::Segment { 
                index: segidx as u16, 
                data: &code[*offset..*offset+len] 
            };
            *offset += len;
        } else if chunk_code == 0x08 {
            *offset += 1;
            contents = ChunkContents::SimpleData { 
                data: &code[*offset..=*offset+2] 
            };
            *offset += 2;
        } else if chunk_code == 0x0E && code[21] == 0xFF {
            *offset += 1;
            contents = ChunkContents::SimpleData { 
                data: &code[*offset..=*offset+6]
            };
            *offset += 6;
        } else if chunk_code <= 0x0D {
            *offset += 1;
            if *offset + 2 > Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize);
            }
            let key_ = get_path_int(&code[*offset..*offset+2]) as u16;
            *offset += 2;
            let len = (chunk_code == 0x09) as usize + (2 *(chunk_code - 0x09) as usize);
            contents = ChunkContents::SimpleRef { 
                key: key_, 
                data: &code[*offset..*offset+len]
            };
            *offset += len;
        } else if chunk_code == 0x0E {
            *offset += 1;
            if *offset + 3 > Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize);
            }
            let key_ = get_int(&code[*offset..*offset+2]) as u16;
            *offset += 2;
            let len = code[*offset] as usize;
            *offset += 1;
            contents = ChunkContents::SimpleRef { 
                key: key_, 
                data: &code[*offset..*offset+len] 
            };
            *offset += len;
        } else if chunk_code == 0x0F && (code[21] & 0x80 > 0 || (code[*offset+1] & 0x80) > 0) {
            if (code[*offset+1] & 0x80) > 0 {
                saved_chunk_code = (saved_chunk_code << 8) + code[*offset+1] as u16;
            }
            *offset += 2;
            if *offset + 3 > Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize)
            }
            let segidx = code[*offset];
            *offset += 1;
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            contents = ChunkContents::Segment { 
                index: segidx as u16, 
                data: &code[*offset..=*offset+len] 
            };
            *offset += len;
        } else if chunk_code == 0x10 {
            *offset += 1;
            contents = ChunkContents::SimpleData { 
                data: &code[*offset..=*offset+3] 
            };
            *offset += 3;
        } else if (0x11..=0x15).contains(&chunk_code) {
            *offset += 1;
            let len = 3 + ((chunk_code == 0x11) as usize) + (2 * (chunk_code as usize - 0x11));
            contents = ChunkContents::SimpleData { 
                data: &code[*offset..=*offset+len] 
            };
            *offset += len;
        } else if chunk_code == 0x16 {
            *offset += 1;
            let key_ = &code[*offset..=*offset+3];
            *offset += 3;
            if *offset >= Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize);
            }
            let len = code[*offset] as usize;
            *offset += 1;
            contents = ChunkContents::LongRef { 
                key: key_,
                data: &code[*offset..=*offset+len] 
            };
            *offset += len;
        } else if chunk_code == 0x17 {
            *offset += 1;
            let key_ = &code[*offset..=*offset+3];
            *offset += 3;
            if *offset + 2 > Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize);
            }
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            contents = ChunkContents::LongRef { 
                key: key_, 
                data: &code[*offset..=*offset+len] 
            };
            *offset += len;
        } else if chunk_code == 0x1B && code[*offset+1] == 0 {
            saved_chunk_code = (saved_chunk_code << 8) + code[*offset+1] as u16;
            *offset += 2;
            let key_ = code[*offset] as u16;
            *offset += 1;
            contents = ChunkContents::SimpleRef { 
                key: key_,
                data: &code[*offset..*offset+4] 
            };
            *offset += 4;
        } else if (0x19..=0x1D).contains(&chunk_code) {
            *offset += 1;
            if *offset > Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize)
            }
            let len = code[*offset] as usize + (((chunk_code == 0x19) as usize) + (2*(chunk_code-0x19) as usize));
            *offset += 1;
            contents = ChunkContents::SimpleData { 
                data: &code[*offset..*offset+len] 
            };
            *offset += len as usize;
        } else if chunk_code == 0x1E {
            *offset += 1;
            if *offset >= Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize)
            }
            let ref_len = code[*offset] as usize;
            *offset += 1;
            let key_ = &code[*offset..=*offset+ref_len];
            *offset += ref_len;

            if *offset >= Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize)
            }
            let len = code[*offset] as usize;
            *offset += 1;
            contents = ChunkContents::LongRef { 
                key: key_, 
                data: &code[*offset..=*offset+len] 
            };
            *offset += len;
        } else if chunk_code == 0x1F {
            *offset += 1;
            if *offset >= Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize)
            }
            let ref_len = code[*offset] as usize;
            *offset += 1;
            let key_ = &code[*offset..=*offset+ref_len];
            *offset += ref_len;
            if *offset + 2 > Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize)
            }
            let len = get_int(&code[*offset..*offset+2]);
            *offset += 2;
            contents = ChunkContents::LongRef { 
                key: key_, 
                data: &code[*offset..=*offset+len] 
            };
            *offset += len;
        } else if chunk_code == 0x20 {
            *offset += 1;
            if *offset >= Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize)
            }
            if code[*offset] == 0xFE {
                *offset += 1;
                contents = ChunkContents::Push { 
                    key: &code[*offset..*offset+8] 
                };
                *offset += 8;
            } else {
                contents = ChunkContents::Push { 
                    key: &code[*offset..=*offset] 
                };
                *offset += 1;
            }
        } else if chunk_code == 0x23 {
            *offset += 1;
            if *offset >= Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize)
            }
            let len = code[*offset] as usize;
            *offset += 1;
            contents = ChunkContents::SimpleData { 
                data: &code[*offset..=*offset+len] 
            };
            *offset += len;
        } else if chunk_code == 0x28 {
            *offset += 1;
            contents = ChunkContents::Push { 
                key: &code[*offset..*offset+2] 
            };
            *offset += 2;
        } else if chunk_code == 0x30 {
            *offset += 1;
            contents = ChunkContents::Push { 
                key: &code[*offset..*offset+3] 
            };
            *offset += 3;
        } else if chunk_code == 0x38 {
            *offset += 1;
            if *offset >= Page::SIZE as usize {
                return Err(ParseErr::DataExceedsSectorSize);
            }
            let len = code[*offset] as usize;
            *offset += 1;
            contents = ChunkContents::Push { 
                key: &code[*offset..*offset+len] 
            };
            *offset += len;
        } else if chunk_code == 0x3d || chunk_code == 0x40 {
            contents = ChunkContents::Pop;
            *offset += 1;
        } else if chunk_code == 0x80 {
            *offset += 1;
        } else {
            return Err(ParseErr::UnrecognizedOpcode(chunk_code));
        };

        return Ok(Chunk::new(
                saved_offset,
                saved_chunk_code, 
                contents,
                delayed))
    }
}
impl fmt::Display for Chunk<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.contents {
        ChunkContents::Push { key } => {
            write!(f, "push:{:?}::data:NA::size:{:?}::ins:{:x}", 
                key,
                key.len(),
                self.opcode)
        },
        ChunkContents::SimpleData { data } => {
            write!(f, "simple:NA::data:{:?}::size:{}::ins:{:x}",
                data,
                data.len(),
                self.opcode)
        }
        ChunkContents::SimpleRef { key, data } => {
            write!(f, "reference:{:?}::ref_data:{:?}::size:{}::ins:{:x}", 
                key,
                data,
                data.len(),
                self.opcode)
        },
        ChunkContents::LongRef { key, data } => {
            write!(f, "reference:{:?}::ref_data:{:?}::size:{}::ins:{:x}",
                key,
                data,
                data.len(),
                self.opcode)
        },
        ChunkContents::Segment { index, data } => {
            write!(f, "segment:{}::data:{:?}::size:{}::ins:{:x}",
                index,
                data,
                data.len(),
                self.opcode)
        },
        ChunkContents::Pop => {
            write!(f, "pop:NA")
        },
        ChunkContents::Noop => {
            write!(f, "noop:NA")
        }
    }
    }
}
//         InstructionType::DataSimple => {
//             write!(f, "path:{:?}::reference:na::simple_data:{:?}::size:{}::ins:{:x}", 
//                  self.path.components,
//                  self.data.unwrap(),
//                  self.data.unwrap().length,
//                  self.opcode)
//         }
//         InstructionType::RefLong => {
//             write!(f, "path:{:?}::reference:{:?}::ref_data:{:?}::size:{}::ins:{:x}", 
//                  self.path.components,
//                  self.ref_data.unwrap(),
//                  self.data.unwrap(),
//                  self.data.unwrap().length,
//                  self.opcode)
//         }
//         InstructionType::PathPush => {
//             write!(f, "path:{:?}::reference:PUSH::ref_data:{:?}::size:{}::ins:{:x}", 
//                  self.path.components,
//                  self.data.unwrap(),
//                  self.data.unwrap().length,
//                  self.opcode)
//         }
//         InstructionType::PathPop => {
//             write!(f, "path:{:?}::reference:POP::ref_data:None::size:{}::ins:{:x}", 
//                  self.path.components,
//                  0,
//                  self.opcode)
//         }
//         InstructionType::Noop => {
//             write!(f, "path:{:?}::reference:NOOP::ref_data:None::size:{}::ins:{:x}", 
//                  self.path.components,
//                  0,
//                  self.opcode)
//         }
//     }
//     }
// }
