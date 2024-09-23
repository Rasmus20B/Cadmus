use std::{ffi::OsString, fs::{write, File, OpenOptions}, io::{BufReader, BufWriter, Read, Seek, Write}, ops::DerefMut, path::Path, thread::current};

use crate::{diff::DiffCollection, fm_format::chunk::{Chunk, ChunkType, InstructionType}, fm_io::block::Block, staging_buffer::{self, DataStaging}, util::{dbcharconv::{self, encode_text}, encoding_util::{fm_string_decrypt, fm_string_encrypt, get_int, get_path_int, put_int, put_path_int}}};

use color_print::cprint;
use rayon::iter::Zip;
use super::path::HBAMPath;

pub struct HBAMFile {
    reader: BufReader<File>,
    writer: BufWriter<File>,
    cursor: usize,
    loaded_block: Option<Block>,
    block_buffer: DataStaging,
    staging_buffer: DataStaging,
}

impl HBAMFile {
    pub fn new(path: &Path) -> Self {
        Self {
            writer: BufWriter::new(
                OpenOptions::new()
                .write(true)
                .open(&path)
                .expect("Unable to open file.")),
            reader: BufReader::new(File::open(&path).expect("Unable to open file.")),
            cursor: 0,
            loaded_block: None,
            block_buffer: DataStaging::new(),
            staging_buffer: DataStaging::new(),
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
            let mut bin_chunk = self.emit_binary_chunk(
                chunk_wrapper.chunk(), 
                match chunk_wrapper {
                    ChunkType::Unchanged(..) => &in_buffer,
                    ChunkType::Modification(..) => data_store
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
        let out_buffer = self.emit_binary_block(&block, data_store).expect("Unable to emit binary representation of block.");
        // TODO: Block overflow must be tracked so indexes can be changed when required.
        self.writer.seek(std::io::SeekFrom::Start(4096 * block.index as u64)).expect("Could not seek into file.");
        if self.writer.write(&out_buffer).expect("Unable to write to file.") != 4096 {
            println!("DIDNT WRITE THE WHOLE BUFFER TBH");
        }
        self.writer.flush().unwrap();
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
                    } else if *hbam_path <= chunk.path {
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
                    } else if *hbam_path <= chunk.path {
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

    pub fn write_dir_chunks_json(&mut self, path: &HBAMPath) {
        let (leaf, _buffer) = self.get_leaf_with_buffer(path);
        let json = serde_json::to_string_pretty(&leaf.chunks).expect("Unable to serialize chunks.");
        write("table_chunks.json", json).expect("Unable to write chunks to json file.");
    }
    
    pub fn write_all_chunks_json(&mut self) {
        unimplemented!()
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
                    } else if *hbam_path <= chunk.path {
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

    fn goto_directory(&mut self, path: &HBAMPath) -> Result<(), String> {
        self.loaded_block = Some(self.get_leaf(path));
        loop {
            for offset in 0..self.loaded_block.as_ref().unwrap().chunks.len() {
                let chunk = self.loaded_block.as_ref().unwrap().chunks[offset].chunk();
                if chunk.path == *path {
                    self.cursor = offset;
                    return Ok(())
                } else if chunk.path > *path {
                    return Err(format!("Directory {:?} not found.", path));
                }
            }
            self.loaded_block = Some(self.get_leaf_n(self.loaded_block.as_ref().unwrap().next as u64));
        }
    }

    fn get_kv(&mut self, key: u16) -> Result<ChunkType, String> {
        let dir_path = self.loaded_block.as_ref().unwrap().chunks[self.cursor].chunk().path.clone();
        loop {
            for offset in self.cursor..self.loaded_block.as_ref().unwrap().chunks.len() {
                let wrapper = &self.loaded_block.as_ref().unwrap().chunks[offset];
                let chunk = wrapper.chunk();
                if chunk.ref_simple == Some(key) {
                    if dir_path == chunk.path {
                        return Ok(wrapper.clone());
                    }
                } else if chunk.path > dir_path {
                    return Err(format!("Key {} not found in directory {:?}", key, dir_path));
                }
            }
            self.loaded_block = Some(self.get_leaf_n(self.loaded_block.as_ref().unwrap().next as u64));
        }
    }

    fn set_kv(&mut self, key: u16, data: &[u8]) -> Result<(), String> {
        let dir_path = self.loaded_block.as_ref().unwrap().chunks[self.cursor].chunk().path.clone();
        loop {
            for offset in self.cursor..self.loaded_block.as_ref().unwrap().chunks.len() {
                let wrapper = &mut self.loaded_block.as_mut().unwrap().chunks[offset];
                let chunk = wrapper.chunk_mut();
                if chunk.ref_simple == Some(key) {
                    if dir_path == chunk.path {
                        let location = self.staging_buffer.store(data.to_vec());
                        chunk.data = Some(location);
                    }
                } else if chunk.path > dir_path {
                    return Err(format!("Key {} not found in directory {:?}", key, dir_path));
                }
            }
            self.loaded_block = Some(self.get_leaf_n(self.loaded_block.as_ref().unwrap().next as u64));
        }
    }

    fn set_long_kv(&mut self, key: Vec<u8>) -> Result<(), String> {
        let dir_path = self.loaded_block.as_ref().unwrap().chunks[self.cursor].chunk().path.clone();
        loop {
            for offset in self.cursor..self.loaded_block.as_ref().unwrap().chunks.len() {
                let wrapper = &self.loaded_block.as_ref().unwrap().chunks[offset];
                let chunk = wrapper.chunk();
                if let Ok(key) = chunk.ref_data.unwrap().lookup_from_buffer(&self.staging_buffer.buffer) {
                    if dir_path == chunk.path {
                    }
                } else if chunk.path > dir_path {
                    return Err(format!("Key {:?} not found in directory {:?}", key, dir_path));
                }
            }
            self.loaded_block = Some(self.get_leaf_n(self.loaded_block.as_ref().unwrap().next as u64));
        }
    }
    pub fn get_directory_bounds<'a>(&self, leaf: &'a Block, path: &HBAMPath) -> (usize, usize) {
        let mut full = leaf.chunks
            .iter()
            .map(|wrapper| wrapper.chunk())
            .enumerate()
            .skip_while(|c| c.1.path != *path)
            .filter(|c| c.1.path == *path)
            .skip(2);
        (full.nth(0).unwrap().0, full.last().unwrap().0)
    }

    pub fn get_keyref<'a>(&self, directory: &'a [ChunkType], key: u16) -> Option<&'a ChunkType> {
        for wrapper in directory {
            let chunk = wrapper.chunk();
            if chunk.ref_simple == Some(key as u16) {
                return Some(wrapper);
            }
        }
        None
    }
    
    pub fn get_keyref_mut<'a>(&self, directory: &'a mut [ChunkType], key: u16) -> Option<&'a mut ChunkType> {
        for wrapper in directory {
            let chunk = wrapper.chunk_mut();
            if chunk.ref_simple == Some(key as u16) {
                return Some(wrapper);
            }
        }
        None
    }

    pub fn get_keyref_by_path_mut<'a>(&self, leaf: &'a mut Block, directory: &HBAMPath, key: u16) -> Option<&'a mut ChunkType> {
        for wrapper in &mut leaf.chunks {
            let chunk = wrapper.chunk_mut();
            if chunk.path == *directory && chunk.ref_simple == Some(key) {
                return Some(wrapper);
            }
        }
        None
    }

    pub fn commit_table_changes(&mut self, diffs: &DiffCollection, data_store: &mut DataStaging) -> Vec<Block> {
        let mut output_blocks = vec![];
        let (mut table_leaf, buffer) = self.get_leaf_with_buffer(&HBAMPath::new(vec!["3", "16"]));
        let meta_search_path = HBAMPath::new(vec!["3", "16", "1", "1"]);
        let (meta_start, meta_end) = self.get_directory_bounds(&table_leaf, &meta_search_path);

        for object in &diffs.modified {
            let mut double_encoded = encode_text(&object.name);
            double_encoded.append(&mut vec![0, 0, 0]);
            let double_encoded_location = data_store.store(double_encoded);
            for ref mut wrapper in table_leaf.chunks[meta_start..=meta_end].iter_mut() {
                let inner_read = wrapper.chunk();
                if inner_read.path != HBAMPath::new(vec!["3", "16", "1", "1"]) {
                    continue;
                }
                let buf = match wrapper {
                    ChunkType::Modification(chunk) => chunk.data.unwrap().lookup_from_buffer(&data_store.buffer).unwrap(),
                    ChunkType::Unchanged(chunk) => chunk.data.unwrap().lookup_from_buffer(&buffer).unwrap(),
                };
                let n = get_int(&buf[buf[0] as usize..]) + 127;
                let chunk = wrapper.chunk_mut();
                let index_location = data_store.store(buf.clone());
                if n == object.id {
                    chunk.ref_data = Some(double_encoded_location);
                    chunk.data = Some(index_location);
                    let new = ChunkType::Modification(wrapper.chunk().clone());
                    *wrapper.deref_mut() = new;
                }
            }

            for ref mut wrapper in &mut table_leaf.chunks {
                let inner_read = wrapper.chunk();
                if inner_read.path != HBAMPath::new(vec!["3", "16", "1"]) || inner_read.ref_simple != Some(252) {
                    continue;
                }
                let buf = match wrapper {
                    ChunkType::Modification(chunk) => chunk.data.unwrap().lookup_from_buffer(&data_store.buffer).unwrap(),
                    ChunkType::Unchanged(chunk) => chunk.data.unwrap().lookup_from_buffer(&buffer).unwrap(),
                };
                let n = get_int(&buf[buf[0] as usize..]) + 1;
                let mut new = put_path_int(n as u32);
                new.insert(0, new.len() as u8);
                let inner_write = wrapper.chunk_mut();
                inner_write.data = Some(data_store.store(new));
                let new = ChunkType::Modification(inner_write.clone());
                **wrapper = new;
                break;

            }

            let (store_start, store_end) = self.get_directory_bounds(&table_leaf, &HBAMPath::new(vec!["3", "16", "5", &object.id.to_string()]));

            let name_chunk = self.get_keyref_mut(&mut table_leaf.chunks[store_start..=store_end], 16);
            if let Some(wrapper) = name_chunk {
                let inner = wrapper.chunk_mut();
                inner.data = Some(data_store.store(fm_string_encrypt(object.name.clone())));
                *wrapper = ChunkType::Modification(inner.clone());
            }

            let change_chunk = self.get_keyref_mut(&mut table_leaf.chunks[store_start..=store_end], 252);
            if let Some(wrapper) = change_chunk {
                let buf = match wrapper {
                    ChunkType::Modification(chunk) => chunk.data.unwrap().lookup_from_buffer(&data_store.buffer).unwrap(),
                    ChunkType::Unchanged(chunk) => chunk.data.unwrap().lookup_from_buffer(&buffer).unwrap(),
                };
                let inner = wrapper.chunk_mut();
                let n = get_int(&buf[buf[0] as usize..]) + 1;
                let mut new = put_path_int(n as u32);
                new.insert(0, new.len() as u8);
                inner.data = Some(data_store.store(new));
                *wrapper = ChunkType::Modification(inner.clone());
            }

            let (mut meta_leaf, buffer) = self.get_leaf_with_buffer(&HBAMPath::new(vec!["2"]));
            let metachunk1 = &mut self.get_keyref_by_path_mut(&mut meta_leaf, &HBAMPath::new(vec!["2"]), 8).expect("Unable to get keyref.");
            let inner = metachunk1.chunk_mut();
            let mut copy = inner.data.unwrap().lookup_from_buffer(&buffer).expect("Unable to find offset in buffer.");
            copy[42] += 1;
            copy[157] += 1;
            inner.data = Some(data_store.store(copy));
            **metachunk1 = ChunkType::Modification(inner.clone());

            let metachunk2 = &mut self.get_keyref_by_path_mut(&mut meta_leaf, &HBAMPath::new(vec!["2"]), 9).expect("Unable to get keyref.");
            let inner = metachunk2.chunk_mut();
            let mut copy = inner.data.unwrap().lookup_from_buffer(&buffer).expect("Unable to find offset in buffer.");
            copy[36] += 1;
            inner.data = Some(data_store.store(copy));
            **metachunk2 = ChunkType::Modification(inner.clone());

            let (mut occurrence_leaf, buffer) = self.get_leaf_with_buffer(&HBAMPath::new(vec!["2"]));
            let occurencechunk1 = &mut self.get_keyref_by_path_mut(&mut meta_leaf, &HBAMPath::new(vec!["2"]), 8).expect("Unable to get keyref.");

        }
        output_blocks.push(table_leaf);
        output_blocks
    }

    pub fn commit_changes(&mut self, diffs: &DiffCollection, data_store: &mut DataStaging) {
        let new_leaves = self.commit_table_changes(diffs, data_store);
        for leaf in new_leaves {
            self.write_node(&leaf, data_store).expect("Unable to write table block to file.");
        }
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




