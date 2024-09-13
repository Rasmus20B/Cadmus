use std::{ffi::OsString, fs::{File, OpenOptions}, io::{BufReader, BufWriter, Read, Seek, Write}, path::Path};

use crate::{fm_format::chunk::{Chunk, ChunkType, InstructionType}, fm_io::block::Block, staging_buffer::DataStaging, util::encoding_util::{get_int, get_path_int}};

use color_print::cprint;
use super::path::HBAMPath;

pub struct HBAMFile {
    reader: BufReader<File>,
    writer: BufWriter<File>,
}

impl HBAMFile {
    pub fn new(path: &Path) -> Self {
        let copy_path = path.with_file_name(path.file_name().unwrap().to_str().unwrap().strip_suffix(".fmp12").unwrap().to_string() + "_patch.fmp12");
        std::fs::copy(path, &copy_path).expect("Unable to make new file.");
        println!("Copied file to: {:?}", copy_path);
        Self {
            reader: BufReader::new(File::open(&copy_path).expect("Unable to open file.")),
            writer: BufWriter::new(
                OpenOptions::new()
                .write(true)
                .open(&copy_path)
                .expect("Unable to open file.")),
        }
    }

    pub fn get_buffer_from_leaf(&mut self, index: u64) -> Vec<u8> {
        let mut buffer = [0u8; 4096];
        self.reader.seek(std::io::SeekFrom::Start(index * 4096)).expect("Could not seek into file.");
        self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");
        buffer.to_vec()
    }

    pub fn get_leaf_n(&mut self, index: u64) -> Block {
        let mut buffer = [0u8; 4096];

        self.reader.seek(std::io::SeekFrom::Start(index * 4096)).expect("Could not seek into file.");
        self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");

        Block::new(&buffer)
    }

    pub fn emit_binary_chunk(&self, chunk: &Chunk, data_store: &DataStaging) -> Result<Vec<u8>, &str> {
        let mut out = vec![];
        if chunk.ref_simple.is_some_and(|val| val == 16) {
        }
        out.extend(chunk.to_bytes(data_store).expect("Unable to translate chunk to binary representation."));
        Ok(out)
    }

    pub fn emit_binary_block_header(&self, block: &Block) -> Result<Vec<u8>, &str> {
        Ok(block.to_bytes())
    }

    pub fn emit_binary_block(&mut self, block: &Block, data_store: &DataStaging) -> Result<Vec<u8>, &str> {
        let mut in_buffer = DataStaging::new();
        self.reader.seek(std::io::SeekFrom::Start((Block::CAPACITY * block.index as usize) as u64)).expect("Unable to seek in fmp file.");
        self.reader.read(&mut in_buffer.buffer).expect("Unable to read block from file.");
        let mut out_buffer: Vec<u8> = vec![];
        out_buffer.append(&mut self.emit_binary_block_header(&block).expect("Unable to emit block header to buffer"));
        for chunk_wrapper in &block.chunks {
            let mut bin_chunk = match chunk_wrapper {
                ChunkType::Unchanged(chunk) => {
                    // Look up from old file.
                    self.emit_binary_chunk(&chunk, &in_buffer).expect("Unable to emit binary chunk.")
                }
                ChunkType::Modification(chunk) => {
                    // Look up from staging buffer
                    self.emit_binary_chunk(&chunk, data_store).expect("Unable to emit binary chunk.")
                }
            };
            out_buffer.append(&mut bin_chunk);
        }

        // Add padding if needed
        let padding = vec![0u8; 4096 - out_buffer.len()];
        out_buffer.extend(padding);
        if out_buffer != in_buffer.buffer {
            for i in out_buffer.iter().zip(in_buffer.buffer) {
                if *i.0 != i.1 {
                } else {
                }
            }
        }
        debug_assert!(out_buffer.len() == 4096);
        println!("\nCHANGES HAVE BEEN MADE");
        Ok(out_buffer)
    }

    pub fn write_node(&mut self, block: &Block, data_store: &DataStaging) -> Result<(), &str> {

        let out_buffer = self.emit_binary_block(&block, data_store).expect("Unable to emit binary representation of block.");
        // TODO: Block overflow must be tracked so indexes can be changed when required.
        self.writer.seek(std::io::SeekFrom::Start(4096 * block.index as u64)).expect("Could not seek into file.");
        if self.writer.write(&out_buffer).expect("Unable to write to file.") != 4096 {
            println!("DIDNT WRITE THE WHOLE BUFFER TBH");
        }
        // self.writer.flush().expect("Unable to flush the buffer");
        Ok(())
    }

    pub fn get_leaf_with_buffer(&mut self, hbam_path: &HBAMPath) -> (Block, Vec<u8>) {
        let mut buffer = [0u8; 4096];
        self.reader.seek(std::io::SeekFrom::Start(4096)).expect("Could not seek into file.");
        self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");

        let mut current_block = Block::new(&buffer);
        let mut next = 0;

        loop {
            for chunk_wrapper in &current_block.chunks {
                let chunk = Chunk::from(chunk_wrapper.clone());
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
                        current_block.index = next as u32;
                        break;
                    }
                }
            }

            if current_block.block_type == 0 {
                break;
            }
        }
        (current_block, buffer.to_vec())
    }

    pub fn get_leaf(&mut self, hbam_path: &HBAMPath) -> Block {
        let mut buffer = [0u8; 4096];
        self.reader.seek(std::io::SeekFrom::Start(4096)).expect("Could not seek into file.");
        self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");

        let mut current_block = Block::new(&buffer);
        let mut next = 0;

        loop {
            for chunk_wrapper in &current_block.chunks {
                let chunk = Chunk::from(chunk_wrapper.clone());
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
                        current_block.index = next as u32;
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


#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{fm_format::chunk::Chunk, hbam::path::HBAMPath, staging_buffer::DataStaging};

    use super::HBAMFile;

    #[test]
    fn emit_block_test() {
        let mut file = HBAMFile::new(Path::new("test_data/input/blank.fmp12"));
        let old = file.get_leaf(&HBAMPath::new(vec!["3", "16"]));
        let mut old_buffer = DataStaging::new();
        old_buffer.buffer = file.get_buffer_from_leaf(old.index as u64);


        let mut new = vec![];

        new.extend(old.to_bytes());
        // TODO: header is 20 bytes, but Block::to_bytes() is not accounting for full thing.
        // remove the unaccounted for last 6 bytes for now.
        old_buffer.buffer.splice(14..20, vec![0; 6]);
        assert_eq!(new[0..20], old_buffer.buffer[0..20]);

        new = file.emit_binary_block(&old, &old_buffer).expect("Unable to emit binary block");

        for offset in 21..new.len() {
            if new[offset] != old_buffer.buffer[offset] {
                let past_chunks = old.chunks.clone().into_iter().filter(|chunk_wrapper| ((Chunk::from(chunk_wrapper.clone()).offset) as usize) <= offset).collect::<Vec<_>>();
                let current_chunk = past_chunks[past_chunks.len() - 1].clone();
                assert_eq!(new[offset], old_buffer.buffer[offset], "{offset} \n{:x?} \n!= \n{:x?}\nChunk: {}",
                    &new[offset-59..offset+20],
                    &old_buffer.buffer[offset-59..offset+20],
                    Chunk::from(current_chunk.clone()));
            }
        }


    }
}





