use std::{fs::File, io::{BufReader, BufWriter, Read, Seek}, path::Path};
use crate::fm_format::chunk::Chunk;
use super::{block::Block, data_location::DataLocation};
use rayon::prelude::*;

pub struct FmpFileHandle {
    read_handle: BufReader<File>,
    write_handle: BufWriter<File>,
    pub blocks: Vec<Block>,
}

impl FmpFileHandle {

    pub(crate) fn get_chunk_payload(&mut self, index: u32) -> Result<Vec<u8>, &str> {
        let mut buffer = [0u8; 4096];
        self.read_handle.seek(std::io::SeekFrom::Start(index as u64 * Block::CAPACITY as u64)).expect("Unable to seek from file.");
        self.read_handle.read_exact(&mut buffer).expect("Unable to read from file.");
        Ok(buffer.to_vec())
    }

    pub(crate) fn fetch_data(&mut self, location: DataLocation) -> Vec<u8> {

        // TODO: Implement cache to call first.
        let block_handle = self.get_chunk(location.chunk)
            .expect("Chunk does not exist.")
                .chunks.get(location.block as usize)
                .expect("Block does not exist.");

        let block_offset = block_handle.offset;

        let buffer = self.get_chunk_payload(location.chunk).expect("Invalid Chunk ID.");
        let start = block_offset + location.offset as u16;
        buffer[start as usize..start as usize +location.length as usize].to_vec()
    }

    pub(crate) fn get_chunk(&self, index: u32) -> Result<&Block, &str> {
        let res = &self.blocks.get(index as usize);

        if res.is_none() {
            return Err("Invalid chunk index.")
        }
        Ok(res.unwrap())
    }

    pub(crate) fn get_chunks(&self) -> &Vec<Block> {
        &self.blocks
    }

    pub fn update_chunk(&mut self, index: usize) -> Result<(), &str> {

        let chunk_payload = self.get_chunk_payload(index as u32).expect("Could not get payload for chunk.");
        let mut ammended_buffer = Vec::new();
        ammended_buffer.extend(self.get_chunk(index as u32).unwrap().to_bytes());

        for instruction in &self.blocks[index].chunks {
            ammended_buffer.extend(instruction.to_bytes(&chunk_payload).expect("Unable to encode instruction."));
        }

        self.write_handle.seek(std::io::SeekFrom::Start(index as u64 * Block::CAPACITY as u64))
            .expect("Unable to seek from file.");



        Ok(())
    }

    pub fn get_data_from_instruction(&mut self, chunk_index: u32, instruction: Chunk) -> Result<Vec<u8>, &str> {
        let data_bind = instruction.data.unwrap();
        let buffer = self.get_chunk_payload(chunk_index).expect("Invalid Chunk ID.");
        Ok(buffer[data_bind.offset as usize..data_bind.offset as usize +data_bind.length as usize].to_vec())
    }

    pub fn from_path(path: &Path) -> Self {

        let mut read_handle_ = BufReader::new(File::open(path).expect("[-] Unable to open file."));
        let write_handle_ = BufWriter::new(File::open(path).expect("[-] Unable to open file."));
        let mut chunks_ = vec![];

        let mut buffer = Vec::new();
        read_handle_.read_to_end(&mut buffer).expect("[-] Unable to read from File.");

        // let mut offset = Chunk::CAPACITY;
        let mut chunk_index: u32 = 2;
        // let mut buffer = vec![0u8; 4096];

        let mut offset = Block::CAPACITY;

        // read_handle_.seek(std::io::SeekFrom::Start(offset as u64)).expect("Unable to seek in file.");
        // read_handle_.read_exact(&mut buffer).expect("Unable to read from file.");

        /* The first chunk stores metadata about the rest of the file */
        let meta_chunk = Block::from_bytes(offset, Some(chunk_index), &buffer[offset..offset+4096]);
        /* total chunks is stored at the same offset as the "next" index in a regular chunk */
        let n_chunks = meta_chunk.next;

        chunks_.resize(n_chunks as usize + 1, Block::default());
        // let mut chunks = buffer.chunks(Chunk::CAPACITY).skip(2).map(|segment| {
        //     let res = Chunk::header_from_bytes(&segment);
        //     res
        // })
        // .collect::<Vec<_>>();
        //
        // chunk_index = chunks.binary_search_by(|x| x.next.cmp(&(0))).expect("Less chunks found than specified in header.") as u32;
        // while chunk_index != 0 {
        //     print!("index: {} :: ", chunk_index);
        //     chunks[chunk_index as usize].index = chunk_index;
        //     let prev = chunks[chunk_index as usize].previous;
        //     println!("chunk: {}, previous: {}", chunks[chunk_index as usize].index, chunks[chunk_index as usize].previous);
        //     chunk_index = chunks.binary_search_by(|x| x.next.cmp(&(prev))).expect("Less chunks found than specified in header.") as u32;
        //     chunk_index = chunks[chunk_index as usize].previous;
        // }
        chunk_index = 2;
        while chunk_index != 0 {
            offset = chunk_index as usize * Block::CAPACITY;
            // read_handle_.seek(std::io::SeekFrom::Start(offset as u64)).expect("Unable to seek in file.");
            // read_handle_.read_exact(&mut buffer).expect("Unable to read from file.");
            let mut current_chunk = Block::header_from_bytes(&buffer[offset..offset+4096]);
            current_chunk.index = chunk_index;
            let next = current_chunk.next;
            chunks_[chunk_index as usize] = current_chunk;
            chunk_index = next;
        }

        chunks_.par_iter_mut().for_each(|chunk| {
            let offset = (chunk.index as usize) * Block::CAPACITY;
            chunk.read_chunks(&buffer[offset..offset+Block::CAPACITY]).expect("Unable to read block.");
        });

        Self {
            read_handle: read_handle_,
            write_handle: write_handle_,
            blocks: chunks_,
        }
    }
}

