use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::sync::RwLock;

use crate::{fm_core::{component::FMTable, file_repr::{FmpFileView, FmpObjectManager}}, fm_io::{chunk::Chunk, data_location::DataLocation}};
use super::block::InstructionType;

pub fn read_chunk(chunk: &Chunk, objects: &RwLock<FmpObjectManager>) {
    chunk.blocks.par_iter().enumerate().for_each(|(i, block)| { 
        match &block.path.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice() {
            ["3", "16", "5", x] => {
                if block.ctype == InstructionType::RefSimple {
                    match block.ref_simple.unwrap_or(0) {
                        16 => { 
                            let data_uw = block.data.unwrap();
                            objects.write().expect("Unable to open write lock to objects.").tables.insert(
                                x.parse().unwrap(),
                                FMTable {
                                    table_name: DataLocation::new(
                                                    chunk.index,
                                                    i as u16,
                                                    (data_uw.offset - block.offset as u32) as u8,
                                                    data_uw.length as u8)
                                },
                            );
                        },
                        _ => {},
                    };
                }
            },
            _ => {}
        }
    })
}

