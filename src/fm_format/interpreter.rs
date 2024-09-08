use crate::{fm_core::{component::FMTableRef, file_repr::FmpFileView}, fm_io::{chunk::Chunk, data_offset::ChunkOffset}};
use super::instruction::InstructionType;

pub fn read_chunk(chunk: &Chunk, file: &mut FmpFileView) {
    for (i, instruction) in chunk.instructions.iter().enumerate() {
        match &instruction.path.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice() {
            ["3", "16", "5", x] => {
                if instruction.ctype == InstructionType::RefSimple {
                    match instruction.ref_simple.unwrap_or(0) {
                        16 => { 
                            println!("instruction: {} / {} for chunk {}", i, chunk.instructions.len(), chunk.index);
                            file.tables.insert(
                                x.parse().unwrap(),
                                FMTableRef {
                                    table_name: ChunkOffset::new(chunk.index, i as u32)
                                },
                            );
                        },
                        _ => {},
                    };
                }
            },
            _ => {}
        }
    }
}

