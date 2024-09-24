use std::path::Path;

use crate::{fm_format::chunk::ChunkType, hbam::btree::HBAMCursor};

use super::{btree::HBAMFile, path::HBAMPath};


pub struct HBAMInterface {
    inner: HBAMFile,
}

impl HBAMInterface {

    pub fn new(path: &Path) -> Self {
        Self {
            inner: HBAMFile::new(path)
        }
    }

    fn goto_directory(&mut self, path: &HBAMPath) -> Result<(), String> {
        let mut block = self.inner.get_leaf(path);
        loop {
            for offset in 0..block.chunks.len() {
                let chunk = block.chunks[offset].chunk();
                if chunk.path == *path {
                    self.inner.cursor = HBAMCursor { block_index: block.index, chunk_index: offset as u16 };
                    return Ok(())
                } else if chunk.path > *path {
                    return Err(format!("Directory {:?} not found.", path));
                }
            }
            block = self.inner.get_next_leaf().expect("Unable to get next leaf.");
        }
    }

    fn get_kv(&mut self, key: u16) -> Result<ChunkType, String> {
        let mut block = self.inner.get_current_block();
        let mut start = self.inner.cursor.chunk_index;
        println!("Looking for kv in : @ {},{}", self.inner.cursor.block_index, self.inner.cursor.chunk_index);
        let dir_path = block.chunks[self.inner.cursor.chunk_index as usize].chunk().path.clone();
        loop {
            for offset in start as usize..block.chunks.len() {
                let wrapper = &block.chunks[offset];
                let chunk = wrapper.chunk();
                if chunk.ref_simple == Some(key) {
                    if dir_path == chunk.path {
                        return Ok(wrapper.clone());
                    }
                } else if chunk.path > dir_path {
                    return Err(format!("Key {} not found in directory {:?}", key, dir_path));
                }
            }
            block = self.inner.get_next_leaf().expect("Unable to get next leaf.");
            start = 0;
        }
    }

//     fn set_kv(&mut self, key: u16, data: &[u8]) -> Result<(), String> {
//         let dir_path = self.cached_blocks.as_ref().unwrap().chunks[self.cursor].chunk().path.clone();
//         loop {
//             for offset in self.cursor..self.cached_blocks.as_ref().unwrap().chunks.len() {
//                 let ref mut wrapper = &mut self.cached_blocks.as_mut().unwrap().chunks[offset];
//                 let chunk = wrapper.chunk_mut();
//                 if chunk.ref_simple == Some(key) {
//                     if dir_path == chunk.path {
//                         println!("Setting {} to {:?} @ path {:?}", key, data, dir_path);
//                         let location = self.staging_buffer.store(data.to_vec());
//                         chunk.data = Some(location);
//                         **wrapper = ChunkType::Modification(chunk.clone());
//                         return Ok(())
//                     }
//                 } else if chunk.path > dir_path {
//                     return Err(format!("Key {} not found in directory {:?}", key, dir_path));
//                 }
//             }
//             self.cached_blocks = Some(self.load_leaf_n_from_disk(self.cached_blocks.as_ref().unwrap().next as u64));
//         }
//     }
//
//     fn set_long_kv(&mut self, key: &Vec<u8>, data: &[u8]) -> Result<(), String> {
//         let dir_path = self.cached_blocks.as_ref().unwrap().chunks[self.cursor].chunk().path.clone();
//         loop {
//             for offset in self.cursor..self.cached_blocks.as_ref().unwrap().chunks.len() {
//                 let wrapper = &mut self.cached_blocks.as_mut().unwrap().chunks[offset];
//                 let chunk = &mut wrapper.chunk_mut();
//                 if let Ok(key) = chunk.ref_data.unwrap().lookup_from_buffer(&self.staging_buffer.buffer) {
//                     if dir_path == chunk.path {
//                         let key_location = self.staging_buffer.store(key.to_vec());
//                         chunk.ref_data = Some(key_location);
//                         let data_location = self.staging_buffer.store(data.to_vec());
//                         chunk.data = Some(data_location);
//                         *wrapper = ChunkType::Modification(chunk.clone());
//                         return Ok(())
//                     }
//                 } else if chunk.path > dir_path {
//                     return Err(format!("Key {:?} not found in directory {:?}", key, dir_path));
//                 }
//             }
//             self.cached_blocks = Some(self.load_leaf_n_from_disk(self.cached_blocks.as_ref().unwrap().next as u64));
//         }
//     }
//
//     fn set_long_kv_by_data(&mut self, key: &Vec<u8>, data: &[u8]) -> Result<(), String> {
//         let dir_path = self.cached_blocks.as_ref().unwrap().chunks[self.cursor].chunk().path.clone();
//         loop {
//             for offset in self.cursor..self.cached_blocks.as_ref().unwrap().chunks.len() {
//                 let wrapper = &mut self.cached_blocks.as_mut().unwrap().chunks[offset];
//                 if wrapper.chunk().data.is_none() { continue; }
//                 let buffer = match wrapper {
//                     ChunkType::Modification(chunk) => chunk.data.unwrap().lookup_from_buffer(&self.staging_buffer.buffer).unwrap(),
//                     ChunkType::Unchanged(chunk) => chunk.data.unwrap().lookup_from_buffer(&self.block_buffer.buffer).unwrap(),
//                 };
//                 let chunk = wrapper.chunk_mut();
//                 println!("{}", chunk.chunk_to_string(&self.block_buffer.buffer));
//                 if let Some(chunk_data) = chunk.data {
//                     if dir_path == chunk.path && buffer == data {
//                         let key_location = self.staging_buffer.store(key.to_vec());
//                         chunk.ref_data = Some(key_location);
//                         let data_location = self.staging_buffer.store(buffer);
//                         chunk.data = Some(data_location);
//                         let new = chunk.clone();
//                         *wrapper = ChunkType::Modification(new);
//                         return Ok(())
//                     }
//                 } else if chunk.path > dir_path {
//                     return Err(format!("Key {:?} not found in directory {:?}", key, dir_path));
//                 }
//             }
//             self.cached_blocks = Some(self.load_leaf_n_from_disk(self.cached_blocks.as_ref().unwrap().next as u64));
//         }
//     }
//
//     pub fn get_directory_bounds<'a>(&self, leaf: &'a Block, path: &HBAMPath) -> (usize, usize) {
//         let mut full = leaf.chunks
//             .iter()
//             .map(|wrapper| wrapper.chunk())
//             .enumerate()
//             .skip_while(|c| c.1.path != *path)
//             .filter(|c| c.1.path == *path)
//             .skip(2);
//         (full.nth(0).unwrap().0, full.last().unwrap().0)
//     }
//
//     pub fn get_keyref<'a>(&self, directory: &'a [ChunkType], key: u16) -> Option<&'a ChunkType> {
//         for wrapper in directory {
//             let chunk = wrapper.chunk();
//             if chunk.ref_simple == Some(key as u16) {
//                 return Some(wrapper);
//             }
//         }
//         None
//     }
//     
//     pub fn get_keyref_mut<'a>(&self, directory: &'a mut [ChunkType], key: u16) -> Option<&'a mut ChunkType> {
//         for wrapper in directory {
//             let chunk = wrapper.chunk_mut();
//             if chunk.ref_simple == Some(key as u16) {
//                 return Some(wrapper);
//             }
//         }
//         None
//     }
//
//     pub fn get_keyref_by_path_mut<'a>(&self, leaf: &'a mut Block, directory: &HBAMPath, key: u16) -> Option<&'a mut ChunkType> {
//         for wrapper in &mut leaf.chunks {
//             let chunk = wrapper.chunk_mut();
//             if chunk.path == *directory && chunk.ref_simple == Some(key) {
//                 return Some(wrapper);
//             }
//         }
//         None
//     }
// pub fn commit_table_changes(&mut self, diffs: &DiffCollection) -> Vec<Block> {
//         let mut output_blocks = vec![];
//         let (table_leaf, _buffer) = self.get_leaf_with_buffer(&HBAMPath::new(vec!["3", "16"]));
//         // let meta_search_path = HBAMPath::new(vec!["3", "16", "1", "1"]);
//         // let (meta_start, meta_end) = self.get_directory_bounds(&table_leaf, &meta_search_path);
//
//         for object in &diffs.modified {
//             let mut double_encoded = encode_text(&object.name);
//             double_encoded.append(&mut vec![0, 0, 0]);
//
//             let fm_encoded = fm_string_encrypt(&object.name);
//             println!("ID: {}", object.id);
//
//             self.goto_directory(&HBAMPath::new(vec!["3", "16", "5", &object.id.to_string()])).expect("Unable to go to directory.");
//             self.set_kv(16, &fm_encoded).expect("Unable to set keyvalue.");
//         //     let double_encoded_location = data_store.store(double_encoded);
//         //     for ref mut wrapper in table_leaf.chunks[meta_start..=meta_end].iter_mut() {
//         //         let inner_read = wrapper.chunk();
//         //         if inner_read.path != HBAMPath::new(vec!["3", "16", "1", "1"]) {
//         //             continue;
//         //         }
//         //         let n = get_int(&buf[buf[0] as usize..]) + 127;
//         //         let chunk = wrapper.chunk_mut();
//         //         let index_location = data_store.store(buf.clone());
//         //         if n == object.id {
//         //             chunk.ref_data = Some(double_encoded_location);
//         //             chunk.data = Some(index_location);
//         //             let new = ChunkType::Modification(wrapper.chunk().clone());
//         //             *wrapper.deref_mut() = new;
//         //         }
//         //     }
//         //
//         //     for ref mut wrapper in &mut table_leaf.chunks {
//         //         let inner_read = wrapper.chunk();
//         //         if inner_read.path != HBAMPath::new(vec!["3", "16", "1"]) || inner_read.ref_simple != Some(252) {
//         //             continue;
//         //         }
//         //         let buf = match wrapper {
//         //             ChunkType::Modification(chunk) => chunk.data.unwrap().lookup_from_buffer(&data_store.buffer).unwrap(),
//         //             ChunkType::Unchanged(chunk) => chunk.data.unwrap().lookup_from_buffer(&buffer).unwrap(),
//         //         };
//         //         let n = get_int(&buf[buf[0] as usize..]) + 1;
//         //         let mut new = put_path_int(n as u32);
//         //         new.insert(0, new.len() as u8);
//         //         let inner_write = wrapper.chunk_mut();
//         //         inner_write.data = Some(data_store.store(new));
//         //         let new = ChunkType::Modification(inner_write.clone());
//         //         **wrapper = new;
//         //         break;
//         //
//         //     }
//         //
//         //     let (store_start, store_end) = self.get_directory_bounds(&table_leaf, &HBAMPath::new(vec!["3", "16", "5", &object.id.to_string()]));
//         //
//         //     let name_chunk = self.get_keyref_mut(&mut table_leaf.chunks[store_start..=store_end], 16);
//         //     if let Some(wrapper) = name_chunk {
//         //         let inner = wrapper.chunk_mut();
//         //         inner.data = Some(data_store.store(fm_string_encrypt(object.name.clone())));
//         //         *wrapper = ChunkType::Modification(inner.clone());
//         //     }
//         //
//         //     let change_chunk = self.get_keyref_mut(&mut table_leaf.chunks[store_start..=store_end], 252);
//         //     if let Some(wrapper) = change_chunk {
//         //         let buf = match wrapper {
//         //             ChunkType::Modification(chunk) => chunk.data.unwrap().lookup_from_buffer(&data_store.buffer).unwrap(),
//         //             ChunkType::Unchanged(chunk) => chunk.data.unwrap().lookup_from_buffer(&buffer).unwrap(),
//         //         };
//         //         let inner = wrapper.chunk_mut();
//         //         let n = get_int(&buf[buf[0] as usize..]) + 1;
//         //         let mut new = put_path_int(n as u32);
//         //         new.insert(0, new.len() as u8);
//         //         inner.data = Some(data_store.store(new));
//         //         *wrapper = ChunkType::Modification(inner.clone());
//         //     }
//         //
//         //     let (mut meta_leaf, buffer) = self.get_leaf_with_buffer(&HBAMPath::new(vec!["2"]));
//         //     let metachunk1 = &mut self.get_keyref_by_path_mut(&mut meta_leaf, &HBAMPath::new(vec!["2"]), 8).expect("Unable to get keyref.");
//         //     let inner = metachunk1.chunk_mut();
//         //     let mut copy = inner.data.unwrap().lookup_from_buffer(&buffer).expect("Unable to find offset in buffer.");
//         //     copy[42] += 1;
//         //     copy[157] += 1;
//         //     inner.data = Some(data_store.store(copy));
//         //     **metachunk1 = ChunkType::Modification(inner.clone());
//         //
//         //     let metachunk2 = &mut self.get_keyref_by_path_mut(&mut meta_leaf, &HBAMPath::new(vec!["2"]), 9).expect("Unable to get keyref.");
//         //     let inner = metachunk2.chunk_mut();
//         //     let mut copy = inner.data.unwrap().lookup_from_buffer(&buffer).expect("Unable to find offset in buffer.");
//         //     copy[36] += 1;
//         //     inner.data = Some(data_store.store(copy));
//         //     **metachunk2 = ChunkType::Modification(inner.clone());
//         //
//         //     let (mut occurrence_leaf, buffer) = self.get_leaf_with_buffer(&HBAMPath::new(vec!["2"]));
//         //     let occurencechunk1 = &mut self.get_keyref_by_path_mut(&mut meta_leaf, &HBAMPath::new(vec!["2"]), 8).expect("Unable to get keyref.");
//         //
//         }
//         output_blocks.push(self.cached_blocks.clone().unwrap());
//         output_blocks
//     }
//
//     pub fn commit_changes(&mut self, diffs: &DiffCollection) {
//         let new_leaves = self.commit_table_changes(diffs);
//         for leaf in new_leaves {
//             self.write_node(&leaf).expect("Unable to write table block to file.");
//         }
//     }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, path::Path};
    use crate::hbam::{btree::HBAMFile, path::HBAMPath};
    use super::HBAMInterface;

    #[test]
    fn dir_traversal_test() {
        let mut file = HBAMInterface::new(Path::new("test_data/input/blank.fmp12"));

        file.goto_directory(&HBAMPath::new(vec!["3", "16"])).expect("Unable to go to directory.");
        file.goto_directory(&HBAMPath::new(vec!["3", "17"])).expect("Unable to go to directory.");
        assert!(file.inner.cached_blocks.len() == 1);
        assert!(file.inner.cached_blocks.contains_key(&64));

        file.goto_directory(&HBAMPath::new(vec!["6"])).expect("Unable to go to directory.");
        assert!(file.inner.cached_blocks.contains_key(&64)
            && file.inner.cached_blocks.contains_key(&62));
        // Fault line between block 62 and 61.
        file.goto_directory(&HBAMPath::new(vec!["6", "5", "1", "14", "0"])).expect("Unable to go to directory.");
        assert!(file.inner.cached_blocks.contains_key(&64)
            && file.inner.cached_blocks.contains_key(&62)
            && file.inner.cached_blocks.contains_key(&61));
        assert!(file.inner.cached_blocks.len() == 3);
    }

    #[test]
    fn kv_retrieval_test() {
        let mut file = HBAMInterface::new(Path::new("test_data/input/blank.fmp12"));
        file.goto_directory(&HBAMPath::new(vec!["3", "16", "5", "129"])).expect("Unable to go to directory.");
        let buffer = file.inner.get_buffer_from_leaf(file.inner.cursor.block_index as u64);

        let kv = file.get_kv(16).unwrap();
        assert_eq!(kv.chunk().data.unwrap().lookup_from_buffer(&buffer).unwrap(), vec![56, 54, 59, 52, 49]);

        let kv = file.get_kv(800);
        assert!(kv.is_err())
    }
}











