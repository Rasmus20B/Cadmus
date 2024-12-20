use std::{collections::HashMap, fs::{write, File, OpenOptions}, io::{BufReader, BufWriter, Read, Seek, Write}, path::Path};

use super::chunk::{Chunk, ChunkType, InstructionType};
use super::block::Block;
use crate::{staging_buffer::DataStaging, util::encoding_util::get_int};

use super::path::HBAMPath;

type BlockIndex = u32;
type ChunkIndex = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HBAMCursor {
    pub block_index: BlockIndex,
    pub chunk_index: ChunkIndex,
}

impl HBAMCursor {
    pub fn new() -> Self {
        Self {
            block_index: 0, chunk_index: 0,
        }
    }
}

pub struct HBAMFile {
    reader: BufReader<File>,
    writer: BufWriter<File>,
    pub cursor: HBAMCursor,
    pub cached_blocks: HashMap<BlockIndex, Block>,
    pub cached_block_data: HashMap<BlockIndex, [u8; 4096]>,
    pub block_buffer: DataStaging,
    pub staging_buffer: DataStaging,
}

impl HBAMFile {
    pub fn new(path: &Path) -> Self {
        Self {
            writer: BufWriter::new(
                OpenOptions::new()
                .write(true)
                .open(path)
                .expect("Unable to open file.")),
            reader: BufReader::new(File::open(path).expect("Unable to open file.")),
            cursor: HBAMCursor::new(),
            cached_blocks: HashMap::new(),
            block_buffer: DataStaging::new(),
            staging_buffer: DataStaging::new(),
            cached_block_data: HashMap::new(),
        }
    }

    pub fn get_current_path(&self) -> HBAMPath {
        let chunk_id = self.cursor.chunk_index;
        self.get_block_chunks()[chunk_id as usize].chunk().path.clone()
    }

    pub fn get_buffer_from_leaf(&mut self, index: u64) -> Vec<u8> {
        let mut buffer = [0u8; 4096];
        self.reader.seek(std::io::SeekFrom::Start(index * 4096)).expect("Could not seek into file.");
        self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");
        buffer.to_vec()
    }

    pub fn load_leaf_n_from_disk(&mut self, index: u32) -> Result<&Block, String> {
        let mut buffer = [0u8; 4096];
        self.reader.seek(std::io::SeekFrom::Start(index as u64 * 4096)).expect("Could not seek into file.");
        self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");
        self.cached_blocks.insert(index, Block::new_with_index(&buffer, index));
        self.cached_block_data.insert(index, buffer);
        Ok(self.cached_blocks.get(&index).unwrap())
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
        self.reader.read_exact(&mut in_buffer.buffer).expect("Unable to read block from file.");
        let mut out_buffer: Vec<u8> = vec![];
        out_buffer.append(&mut self.emit_binary_block_header(block).expect("Unable to emit block header to buffer"));
        for chunk_wrapper in &block.chunks {
            let mut bin_chunk = self.emit_binary_chunk(
                chunk_wrapper.chunk(), 
                match chunk_wrapper {
                    ChunkType::Unchanged(..) => &in_buffer,
                    ChunkType::Modification(..) => { data_store }
                }).expect("Unable to emit binary chunk.");
            out_buffer.append(&mut bin_chunk);
        }

        // Add padding if needed
        let padding = vec![0u8; Block::CAPACITY - out_buffer.len()];
        out_buffer.extend(padding);
        debug_assert!(out_buffer.len() == Block::CAPACITY);
        Ok(out_buffer)
    }

    pub fn write_node(&mut self, block: &Block, data_store: &DataStaging) -> Result<(), &str> {
        let out_buffer = self.emit_binary_block(block, data_store).expect("Unable to emit binary representation of block.");
        // TODO: Block overflow must be tracked so indexes can be changed when required.
        self.writer.seek(std::io::SeekFrom::Start((Block::CAPACITY * block.index as usize) as u64)).expect("Could not seek into file.");
        if self.writer.write(&out_buffer).expect("Unable to write to file.") != 4096 {
            println!("DIDNT WRITE THE WHOLE BUFFER TBH");
        }
        // self.writer.flush().unwrap();
        println!("Successfully wrote changes to file.");
        Ok(())
    }

    pub fn write_nodes(&mut self, data_store: &DataStaging) -> Result<(), &str> {
        let blocks = std::mem::take(&mut self.cached_blocks);
        for block in blocks {
            self.write_node(&block.1, data_store).expect("Unable to write block to file.");
        }
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
                    } else if *hbam_path <= chunk.path {
                        self.reader.seek(std::io::SeekFrom::Start((next as u64) * 4096_u64)).expect("Could not seek into file.");
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
    
    pub fn get_node(&mut self, hbam_path: &HBAMPath) -> &Block {
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
                    } else if *hbam_path <= chunk.path {
                        self.reader.seek(std::io::SeekFrom::Start((next as u64) * 4096_u64)).expect("Could not seek into file.");
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
        let index = current_block.index;
        self.cached_blocks.entry(current_block.index).or_insert(current_block);
        self.cached_blocks.get(&index).unwrap()
    }

    pub fn write_dir_chunks_json(&mut self, path: &HBAMPath) {
        let (leaf, _buffer) = self.get_leaf_with_buffer(path);
        let json = serde_json::to_string_pretty(&leaf.chunks).expect("Unable to serialize chunks.");
        write("table_chunks.json", json).expect("Unable to write chunks to json file.");
    }
    
    pub fn write_all_chunks_json(&mut self) {
        unimplemented!()
    }

    pub fn get_current_block(&self) -> &Block {
        self.cached_blocks.get(&self.cursor.block_index).unwrap()
    }

    pub fn get_current_block_mut(&mut self) -> &mut Block {
        self.cached_blocks.get_mut(&self.cursor.block_index).unwrap()
    }

    pub fn get_current_block_with_buffer(&mut self) -> (&Block, Vec<u8>) {
        let index = self.cursor.block_index;
        let buffer = self.get_buffer_from_leaf(index as u64);
        let block = self.cached_blocks.get(&index).unwrap();
        (block, buffer)
    }

    pub fn get_current_block_with_buffer_mut(&mut self) -> (&mut Block, Vec<u8>) {
        let index = self.cursor.block_index;
        let buffer = self.get_buffer_from_leaf(index as u64);
        let block = self.cached_blocks.get_mut(&index).unwrap();
        (block, buffer)
    }

    pub fn get_block_chunks(&self) -> &Vec<ChunkType> {
        &self.get_current_block().chunks
    }

    pub fn print_root_block(&mut self) {
        let mut buffer = [0u8; Block::CAPACITY];
        self.reader.seek(std::io::SeekFrom::Start(Block::CAPACITY as u64)).expect("Could not seek into file.");
        self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");
            let leaf = Block::new(&buffer);
            for chunk in leaf.chunks {
                let unwrapped = Chunk::from(chunk);
                // println!("{}", unwrapped);
                let text = unwrapped.chunk_to_string(&buffer);
                println!("{}", text);
            }
    }

    pub fn print_all_chunks(&mut self) {
        let mut buffer = [0u8; Block::CAPACITY];
        self.reader.seek(std::io::SeekFrom::Start(Block::CAPACITY as u64)).expect("Could not seek into file.");
        self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");

        let mut index = 2;

        while index != 0 {
            println!("======================");
            println!("Block: {}", index);
            self.reader.seek(std::io::SeekFrom::Start(index * Block::CAPACITY as u64)).expect("Could not seek into file.");
            self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");
            let leaf = Block::new(&buffer);
            for chunk in leaf.chunks {
                let unwrapped = Chunk::from(chunk);
                // println!("{}", unwrapped);
                let text = unwrapped.chunk_to_string(&buffer);
                println!("{}", text);
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

    pub fn get_leaf(&mut self, hbam_path: &HBAMPath) -> &Block {
        let mut buffer = [0u8; 4096];
        if let std::collections::hash_map::Entry::Vacant(e) = self.cached_blocks.entry(1) {
            self.reader.seek(std::io::SeekFrom::Start(4096)).expect("Could not seek into file.");
            self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");

            e.insert(Block::new(&buffer));
            self.cached_block_data.insert(1, buffer);
        }

        let mut current_block = self.cached_blocks.get_mut(&1).unwrap();
        buffer = *self.cached_block_data.get(&1).unwrap();
        current_block.index = 1;
        let mut next = current_block.next as usize;
        let mut found = false;

        loop {
            for chunk_wrapper in &current_block.chunks {
                let chunk = Chunk::from(chunk_wrapper.clone());
                let n: usize;
                if chunk.data.is_some() {
                    let data_uw = chunk.data.unwrap();
                    n = get_int(&buffer[data_uw.offset as usize..data_uw.offset as usize+data_uw.length as usize]);
                    if chunk.ctype == InstructionType::RefSimple {
                        next = n;
                    } else if *hbam_path <= chunk.path {
                        if let std::collections::hash_map::Entry::Vacant(e) = self.cached_blocks.entry(next as u32) {
                            self.reader.seek(std::io::SeekFrom::Start((next as u64) * 4096_u64)).expect("Could not seek into file.");
                            self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");
                            e.insert(Block::new(&buffer));
                            self.cached_block_data.insert(next as u32, buffer);
                        } 
                        current_block = self.cached_blocks.get_mut(&(next as u32)).unwrap();
                        buffer = *self.cached_block_data.get(&(next as u32)).unwrap();
                        current_block.index = next as u32;
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                next = current_block.next as usize;
                if let std::collections::hash_map::Entry::Vacant(e) = self.cached_blocks.entry(next as u32) {
                    self.reader.seek(std::io::SeekFrom::Start((next as u64) * 4096_u64)).expect("Could not seek into file.");
                    self.reader.read_exact(&mut buffer).expect("Could not read from HBAM file.");
                    e.insert(Block::new(&buffer));
                    self.cached_block_data.insert(next as u32, buffer);
                } 
                current_block = self.cached_blocks.get_mut(&(next as u32)).unwrap();
                buffer = *self.cached_block_data.get(&(next as u32)).unwrap();
                current_block.index = next as u32;

            }

            if current_block.block_type == 0 {
                break;
            }
        }

        let index = current_block.index;
        if !self.cached_blocks.contains_key(&index) {
            self.cached_block_data.insert(index, buffer);
        }
        self.cursor.block_index = index;
        self.cached_blocks.get(&index).unwrap()
    }

    fn load_leaf_from_io(&mut self) -> Block {
        unimplemented!()
    }

    pub fn goto_next_leaf(&mut self) -> Result<&Block, String> {
        debug_assert!(!self.cached_blocks.is_empty());
        let next = self.cached_blocks[&self.cursor.block_index].next;

        if self.cached_blocks.contains_key(&(next)) {
            self.cursor.block_index = next;
            self.cursor.chunk_index = 0;
            return Ok(self.cached_blocks.get(&next).unwrap());
        } else {
            self.load_leaf_n_from_disk(next).expect("Unable to load block from disk.");
            self.cursor.block_index = next;
            self.cursor.chunk_index = 0;
            return Ok(self.cached_blocks.get(&next).unwrap());
        }
    }

    pub fn get_next_leaf(&mut self) -> Result<&Block, String> {
        debug_assert!(!self.cached_blocks.is_empty());
        let next = self.cached_blocks[&self.cursor.block_index].next;

        if self.cached_blocks.contains_key(&(next)) {
            self.cursor.block_index = next;
            self.cursor.chunk_index = 0;
            return Ok(self.cached_blocks.get(&next).unwrap());
        } else {
            self.load_leaf_n_from_disk(next).expect("Unable to load block from disk.");
            self.cursor.block_index = next;
            self.cursor.chunk_index = 0;
            return Ok(self.cached_blocks.get(&next).unwrap());
        }
    }

    pub fn get_next_leaf_mut(&mut self) -> Result<&mut Block, String> {
        debug_assert!(!self.cached_blocks.is_empty());
        let next = self.cached_blocks[&self.cursor.block_index].next;

        if self.cached_blocks.contains_key(&(next)) {
            return Ok(self.cached_blocks.get_mut(&next).unwrap());
        } else {
            self.load_leaf_n_from_disk(next).expect("Unable to load block from disk.");
            return Ok(self.cached_blocks.get_mut(&next).unwrap());
        }
    }

    pub fn get_next_leaf_with_buffer(&mut self) -> Result<(&Block, Vec<u8>), String> {
        debug_assert!(!self.cached_blocks.is_empty());
        let next = self.cached_blocks[&self.cursor.block_index].next;
        let buffer = self.get_buffer_from_leaf(next as u64);

        if self.cached_blocks.contains_key(&(next)) {
            return Ok((self.cached_blocks.get(&next).unwrap(), buffer));
        } else {
            self.load_leaf_n_from_disk(next).expect("Unable to load block from disk.");
            return Ok((self.cached_blocks.get(&next).unwrap(), buffer));
        }
    }

    pub fn get_next_leaf_with_buffer_mut(&mut self) -> Result<(&mut Block, Vec<u8>), String> {
        debug_assert!(!self.cached_blocks.is_empty());
        let next = self.cached_blocks[&self.cursor.block_index].next;
        let buffer = self.get_buffer_from_leaf(next as u64);

        if self.cached_blocks.contains_key(&(next)) {
            return Ok((self.cached_blocks.get_mut(&next).unwrap(), buffer));
        } else {
            self.load_leaf_n_from_disk(next).expect("Unable to load block from disk.");
            return Ok((self.cached_blocks.get_mut(&next).unwrap(), buffer));
        }
    }
}

#[cfg(test)]
mod tests {
    
    
    

    #[test]
    fn emit_block_test() {
        // let mut file = HBAMFile::new(Path::new("test_data/input/blank.fmp12"));
        // let old = file.get_leaf(&HBAMPath::new(vec!["3", "16"]));
        // let mut old_buffer = DataStaging::new();
        // old_buffer.buffer = file.get_buffer_from_leaf(old.index as u64);
        //
        //
        // let mut new = vec![];
        //
        // new.extend(old.to_bytes());
        // // TODO: header is 20 bytes, but Block::to_bytes() is not accounting for full thing.
        // // remove the unaccounted for last 6 bytes for now.
        // old_buffer.buffer.splice(14..20, vec![0; 6]);
        // assert_eq!(new[0..20], old_buffer.buffer[0..20]);
        //
        // new = file.emit_binary_block(&old, &old_buffer).expect("Unable to emit binary block");
        //
        // for offset in 21..new.len() {
        //     if new[offset] != old_buffer.buffer[offset] {
        //         let past_chunks = old.chunks.clone().into_iter().filter(|chunk_wrapper| ((Chunk::from(chunk_wrapper.clone()).offset) as usize) <= offset).collect::<Vec<_>>();
        //         let current_chunk = past_chunks[past_chunks.len() - 1].clone();
        //         assert_eq!(new[offset], old_buffer.buffer[offset], "{offset} \n{:x?} \n!= \n{:x?}\nChunk: {}",
        //             &new[offset-59..offset+20],
        //             &old_buffer.buffer[offset-59..offset+20],
        //             Chunk::from(current_chunk.clone()));
        //     }
        // }
    }

    #[test]
    fn changes_test() {

    }

    #[test]
    fn directory_traversal_test() {

    }
}




