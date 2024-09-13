use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::sync::RwLock;

use crate::{fm_core::{component::FMTable, file_repr::{FmpFileView, FmpObjectManager}}, fm_io::{block::Block, data_location::DataLocation}};
use super::chunk::{Chunk, InstructionType};

pub fn read_chunk(block: &Block, objects: &RwLock<FmpObjectManager>) {
    block.chunks.par_iter().enumerate().for_each(|(i, chunk_wrapper)| { 
        let chunk = Chunk::from(chunk_wrapper.clone());
        match &chunk.path.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice() {
            ["3", "16", "5", x] => {
                if chunk.ctype == InstructionType::RefSimple {
                    match chunk.ref_simple.unwrap_or(0) {
                        16 => { 
                            let data_uw = chunk.data.unwrap();
                            objects.write().expect("Unable to open write lock to objects.").tables.insert(
                                x.parse().unwrap(),
                                FMTable {
                                    table_name: DataLocation::new(
                                                    block.index,
                                                    i as u16,
                                                    (data_uw.offset - chunk.offset as u32) as u8,
                                                    data_uw.length as u8)
                                },
                            );
                        },
                        _ => {},
                    };
                }
            },
            ["3", ..] => {
                // println!("block: {}", block.index);
            }
            _ => {}
        }
    })
}

