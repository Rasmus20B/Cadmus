use std::{fs::File, io::{BufReader, Read, Seek}, path::Path};

use crate::fm_format::instruction::Instruction;

use super::chunk::Chunk;

pub struct FmpFileHandle {
    file_handle: BufReader<File>,
    pub chunks: Vec<Chunk>,
}

impl FmpFileHandle {

    pub(crate) fn get_chunk_payload(&mut self, index: u32) -> Result<Vec<u8>, &str> {
        let mut buffer = [0u8; 4096];
        self.file_handle.seek(std::io::SeekFrom::Start(index as u64 * Chunk::SIZE as u64)).expect("Unable to seek from file.");
        self.file_handle.read_exact(&mut buffer).expect("Unable to read from file.");
        Ok(buffer.to_vec())
    }

    pub(crate) fn get_chunks(&self) -> &Vec<Chunk> {
        &self.chunks
    }

    pub fn get_data_from_instruction(&mut self, chunk_index: u32, instruction: Instruction) -> Result<Vec<u8>, &str> {
        let data_bind = instruction.data.unwrap();
        let buffer = self.get_chunk_payload(chunk_index).expect("Invalid Chunk ID.");
        Ok(buffer[data_bind.offset as usize..data_bind.offset as usize +data_bind.length as usize].to_vec())
    }

    pub fn from_path(path: &Path) -> Self {

        let mut file = BufReader::new(File::open(path).expect("[-] Unable to open file."));
        let mut chunks_ = vec![];

        let mut offset = Chunk::SIZE;
        let mut chunk_index: u32 = 2;
        let mut buffer = vec![0u8; 4096];

        file.seek(std::io::SeekFrom::Start(offset as u64)).expect("Unable to seek in file.");
        file.read_exact(&mut buffer).expect("Unable to read from file.");

        /* The first chunk stores metadata about the rest of the file */
        let meta_chunk = Chunk::from_buffer(offset, chunk_index, &buffer);
        /* total chunks is stored at the same offset as the "next" index in a regular chunk */
        let n_chunks = meta_chunk.next;

        chunks_.resize(n_chunks as usize + 1, Chunk::default());
        while chunk_index != 0 {
            offset = chunk_index as usize * Chunk::SIZE;
            file.seek(std::io::SeekFrom::Start(offset as u64)).expect("Unable to seek in file.");
            file.read_exact(&mut buffer).expect("Unable to read from file.");
            let current_chunk = Chunk::from_buffer(offset, chunk_index, &buffer);
            let next = current_chunk.next;
            chunks_[chunk_index as usize] = current_chunk;
            chunk_index = next;
        }

        Self {
            file_handle: file,
            chunks: chunks_,
        }
    }
}

