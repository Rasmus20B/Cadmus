use std::{fs::File, io::{BufReader, BufWriter, Read, Seek}, path::Path};

use crate::{fm_format::chunk::{Chunk, InstructionType}, fm_io::block::Block, util::encoding_util::{get_int, get_path_int}};

use super::path::HBAMPath;

pub struct HBAMFile {
    reader: BufReader<File>,
    writer: BufWriter<File>,
}

impl HBAMFile {
    pub fn new(path: &Path) -> Self {
        Self {
            reader: BufReader::new(File::open(path).expect("Unable to open file.")),
            writer: BufWriter::new(File::open(path).expect("Unable to open file.")),
        }
    }

    pub fn get_leaf_n(&mut self, index: u64) -> Block {
        let mut buffer = [0u8; 4096];

        self.reader.seek(std::io::SeekFrom::Start(index * 4096)).expect("Could not seek into file.");
        self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");

        Block::new(&buffer)
    }

    pub fn get_leaf(&mut self, hbam_path: &HBAMPath) -> Block {

        let mut buffer = [0u8; 4096];

        self.reader.seek(std::io::SeekFrom::Start(4096)).expect("Could not seek into file.");
        self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");

        let mut current_block = Block::new(&buffer);
        /* basic Vec remove since small number of elements. */
        let mut next = 0;

        loop {
            for chunk in &current_block.chunks {
                let n: usize;
                if chunk.data.is_some() {
                    let data_uw = chunk.data.unwrap();
                    n = get_int(&buffer[data_uw.offset as usize..data_uw.offset as usize+data_uw.length as usize]);
                    if chunk.ctype == InstructionType::RefSimple {
                        next = n;
                    } else if *hbam_path <= HBAMPath::new(chunk.path.clone()) {
                        self.reader.seek(std::io::SeekFrom::Start((next as u64) * 4096 as u64)).expect("Could not seek into file.");
                        self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");
                        current_block = Block::new(&buffer);
                        break;
                    }
                }
            }

            if current_block.block_type == 0 {
                break;
            }
        }
        current_block
    }
}

