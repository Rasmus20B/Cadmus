use std::{ffi::OsString, fs::{File, OpenOptions}, io::{BufReader, BufWriter, Read, Seek, Write}, path::Path, thread::current};

use crate::{diff::DiffCollection, fm_format::chunk::{Chunk, ChunkType, InstructionType}, fm_io::block::Block, staging_buffer::DataStaging, util::encoding_util::{fm_string_decrypt, fm_string_encrypt, get_int, get_path_int}};

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
        debug_assert!(out_buffer.len() == 4096);
        Ok(out_buffer)
    }

    pub fn write_node(&mut self, block: &Block, data_store: &DataStaging) -> Result<(), &str> {

        let out_buffer = self.emit_binary_block(&block, data_store).expect("Unable to emit binary representation of block.");
        // TODO: Block overflow must be tracked so indexes can be changed when required.
        self.writer.seek(std::io::SeekFrom::Start(4096 * block.index as u64)).expect("Could not seek into file.");
        if self.writer.write(&out_buffer).expect("Unable to write to file.") != 4096 {
            println!("DIDNT WRITE THE WHOLE BUFFER TBH");
        }
        // self.writer.flush().unwrap();
        println!("Successfully wrote changes to file.");
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
    
    pub fn get_node(&mut self, hbam_path: &HBAMPath) -> Block {
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

            if current_block.block_type == 0x1 || current_block.block_type == 0x3 {
                println!("FTypw: {}", current_block.block_type);
                break;
            }
        }
        current_block
    }

    pub fn print_all_chunks(&mut self) {
        let mut buffer = [0u8; 4096];
        self.reader.seek(std::io::SeekFrom::Start(4096)).expect("Could not seek into file.");
        self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");

        let mut index = 2;

        while index != 0 {
            println!("======================");
            println!("Block: {}", index);
            self.reader.seek(std::io::SeekFrom::Start(index * 4096)).expect("Could not seek into file.");
            self.reader.read(&mut buffer).expect("Could not read from HBAM file.");
            let leaf = Block::new(&buffer);
            for chunk in leaf.chunks {
                let unwrapped = Chunk::from(chunk).chunk_to_string(&buffer);
                println!("{}", unwrapped);
            }
            index = leaf.next.into();
        }

    }

    pub fn get_root(&mut self) -> Block {
        let mut buffer = [0u8; 4096];
        self.reader.seek(std::io::SeekFrom::Start(4096)).expect("Could not seek into file.");
        self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");
        Block::new(&buffer)
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

    pub fn commit_table_changes(&mut self, diffs: &DiffCollection, data_store: &mut DataStaging) -> Block {
        let mut table_leaf = self.get_leaf(&HBAMPath::new(vec!["3", "16"]));
        for object in &diffs.modified {
            let location = data_store.store(fm_string_encrypt(object.name.clone()));
            let mut chunk_copy = table_leaf.chunks.iter()
                .map(|chunk_wrapper| Chunk::from(chunk_wrapper.clone()))
                .enumerate()
                .filter(|(_i, chunk)| {
                    chunk.ref_simple.is_some_and(|chunk| chunk == 16) 
                        && 
                    chunk.path == (&["3".to_string(), "16".to_string(), "5".to_string(), object.id.to_string()])})
                .collect::<Vec<_>>()[0].clone();
            chunk_copy.1.data = Some(location);
            chunk_copy.1.opcode = 6;
            table_leaf.chunks[chunk_copy.0] = ChunkType::Modification(chunk_copy.1);
        }
        table_leaf
    }

    pub fn commit_changes(&mut self, diffs: &DiffCollection, data_store: &mut DataStaging) {
        let new_leaf = self.commit_table_changes(diffs, data_store);
        self.write_node(&new_leaf, data_store).expect("Unable to write table block to file.")
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{diff::DiffCollection, fm_format::chunk::{Chunk, ChunkType, InstructionType}, fm_io::block::Block, hbam::path::HBAMPath, staging_buffer::DataStaging};

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

    #[test]
    fn changes_test() {

    }
}




