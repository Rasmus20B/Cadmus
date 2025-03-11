
mod error;
mod request;
mod scheduler;

use std::{collections::{HashMap, HashSet}, fs::{File, OpenOptions}, io::{BufReader, BufWriter, Read, Seek, Write}, path::PathBuf};

use error::{Error, Result};

use super::page::header::PageHeader;

pub trait DiskIO {
    fn write_page(&mut self, page_id: usize, data: &[u8;4096]) -> Result<()>;
    fn read_page(&self, page_id: usize) -> Result<[u8;4096]>;
    fn delete_page(&self, page_id: usize) -> Result<()>;
}


pub struct DiskManager {
    pub file_path: PathBuf,
    pub file: File,
    pub page_offsets: HashMap<usize, usize>,
    pub free_slots: Vec<usize>,
}

impl DiskManager {
    pub fn new(file_path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .open(file_path.clone())?;

        Ok(Self {
            file_path,
            free_slots: vec![],
            page_offsets: Self::_populate_offset_table(&file)?,
            file
        })
    }

    fn _populate_offset_table(file: &File) -> Result<HashMap<usize, usize>>{
        let mut reader = BufReader::new(file);
        let mut table = HashMap::new();
        const PAGE_SIZE: u64 = 4096;

        let file_size = file.metadata().unwrap().len();
        println!("Filesize: {file_size}");
        let pages_n = file_size / PAGE_SIZE;

        let root_page = PAGE_SIZE * 1;
        let mut cur = 2;

        let duplicate_check = HashSet::<usize>::new();
        for i in 1..pages_n {
            if duplicate_check.contains(&cur) { return Err(Error::CorruptedFile) }

            table.insert(i as usize, cur * PAGE_SIZE as usize);
            let mut header_buf = [0u8; 20];
            reader.seek(std::io::SeekFrom::Start(cur as u64 *PAGE_SIZE))?;
            reader.read(&mut header_buf)?;
            let header = PageHeader::from_bytes(&header_buf);
            cur = header.next as usize;
        }

        return Ok(table);
    }
}

impl DiskIO for DiskManager {
    fn write_page(&mut self, page_id: usize, data: &[u8;4096]) -> Result<()> {
        let mut writer = BufWriter::new(&mut self.file);
        let Some(position) = self.page_offsets.get(&page_id) else {
            if let Some(slot) = self.free_slots.last() {
                // Write the page to the free slot
                writer.seek(std::io::SeekFrom::Start(*slot as u64))?;
                self.page_offsets.insert(page_id, *slot);
                self.free_slots.pop();
                writer.write(data)?;
                return Ok(())
            }
            // Write the page to the end of the file
            writer.seek(std::io::SeekFrom::End(0))?;
            writer.write(data)?;
            return Ok(())
        }; 

        // Write the page to it's existing location
        writer.seek(std::io::SeekFrom::Start(*position as u64))?;
        writer.write(data)?;
        Ok(())
    }

    fn read_page(&self, page_id: usize) -> Result<[u8; 4096]> {
        todo!()
    }

    fn delete_page(&self, page_id: usize) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::path::PathBuf;

    use super::DiskManager;
    use super::Result;

    #[test]
    fn test_open_file() -> Result<()> {
        let manager = DiskManager::new(PathBuf::from(Path::new("test_data/blank.fmp12")))?;

        println!("pages: {}", manager.page_offsets.len());
        for (id, offset) in manager.page_offsets {
            println!("Page: {id}, offset: {offset}");
        }

        Ok(())

    }
}
