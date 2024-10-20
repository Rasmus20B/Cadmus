use std::{collections::HashMap, path::Path};

use crate::{diff::{DiffCollection, SchemaDiff}, hbam::{btree::HBAMCursor, chunk::{ChunkType, InstructionType}}, schema::{AutoEntry, AutoEntryType, DBObjectStatus, Field, LayoutFM, Relation, RelationComparison, RelationCriteria, Schema, Table, TableOccurrence, Validation, ValidationTrigger}, staging_buffer::DataStaging, util::{dbcharconv::encode_text, encoding_util::{fm_string_decrypt, fm_string_encrypt, get_int, get_path_int, put_int, put_path_int}}};

use super::{btree::HBAMFile, path::HBAMPath};


pub struct HBAMInterface {
    pub inner: HBAMFile,
    staging_buffer: DataStaging,
    block_buffer: Vec<u8>,
}

impl HBAMInterface {

    pub fn new(path: &Path) -> Self {
        Self {
            inner: HBAMFile::new(path),
            staging_buffer: DataStaging::new(),
            block_buffer: vec![],
        }
    }

    pub fn get_tables(&mut self) -> HashMap<usize, Table> {
        let mut result = HashMap::new();
        let mut table_storage_path = HBAMPath::new(vec!["3", "16", "5"]);
        self.goto_directory(&table_storage_path).expect("Unable to go to directory.");
        for x in 129..=255 {
            table_storage_path.components.push(x.to_string());
            if let Ok(()) = self.goto_directory(&table_storage_path) {
                let name = fm_string_decrypt(&self.get_kv_value(16).expect("Unable to get keyvalue"));
                let created_by = fm_string_decrypt(&self.get_kv_value(64513).expect("Unable to get keyvalue"));
                let modified_by = fm_string_decrypt(&self.get_kv_value(64514).expect("Unable to get keyvalue"));
                let mut tmp = Table::new(x);
                tmp.name = name;
                tmp.created_by = created_by;
                tmp.modified_by = modified_by;
                result.insert(x, tmp);
            } 
            table_storage_path.components.pop();
        }
        result
    }

    pub fn get_table_occurrences(&mut self, schema: &mut Schema) {
        let mut table_storage_path = HBAMPath::new(vec!["3", "17", "5"]);
        self.goto_directory(&table_storage_path).expect("Unable to go to directory.");
        for x in 129..=255 {
            table_storage_path.components.push(x.to_string());
            if let Ok(()) = self.goto_directory(&table_storage_path) {
                let definition = &self.get_kv_value(2).expect("Unable to get keyvalue");
                let name = fm_string_decrypt(&self.get_kv_value(16).expect("Unable to get keyvalue"));
                let created_by = fm_string_decrypt(&self.get_kv_value(64513).expect("Unable to get keyvalue"));
                let modified_by = fm_string_decrypt(&self.get_kv_value(64514).expect("Unable to get keyvalue"));
                let mut tmp_occurrence = TableOccurrence::new(x);
                tmp_occurrence.name = name;
                tmp_occurrence.created_by = created_by;
                tmp_occurrence.modified_by = modified_by;
                tmp_occurrence.table_actual = definition[6] as u16 + 128;
                schema.table_occurrences.insert(x, tmp_occurrence);

                table_storage_path.components.push(251.to_string());
                if let Ok(()) = self.goto_directory(&table_storage_path) {
                    let relation_definitions = &self.get_simple_data().expect("Unable to get top-level relationship definition.");
                    if relation_definitions.is_empty() { continue; }
                    let relation_definition = relation_definitions[0].clone();
                    let mut tmp = Relation::new(0);
                    let relation_index = relation_definition[4];
                    tmp.table1 = x as u16;
                    tmp.table2 = relation_definition[2] as u16 + 128;
                    tmp.id = relation_index as usize;
                    schema.relations.insert(relation_index as usize, tmp);
                }
                table_storage_path.components.pop();
            } 
            table_storage_path.components.pop();
        }

        let mut table_storage_path = HBAMPath::new(vec!["3", "251", "5"]);
        for (idx, rel_handle) in schema.relations.iter_mut() {
            table_storage_path.components.push(idx.to_string());
            table_storage_path.components.push(3.to_string());
            if let Ok(()) = self.goto_directory(&table_storage_path) {
                for i in 0..20 {

                if let Ok(definition) = &self.get_kv_value(i) {
                    let comparison_ = match definition[0] {
                        0 => RelationComparison::Equal,
                        1 => RelationComparison::NotEqual,
                        2 => RelationComparison::Greater,
                        3 => RelationComparison::GreaterEqual,
                        4 => RelationComparison::Less,
                        5 => RelationComparison::LessEqual,
                        6 => RelationComparison::Cartesian,
                        _ => unreachable!()
                    };
                    let start1 = 2_usize;
                    let len1 = definition[1] as usize;
                    let start2 = start1 + len1 + 1_usize;
                    let len2 = definition[start1 + len1] as usize;
                    let n1 = get_path_int(&definition[start1..start1 + len1]);
                    let n2 = get_path_int(&definition[start2..start2 + len2]);
                    let field1_ = n1 as u16 - 128;
                    let field2_ = n2 as u16 - 128;
                    rel_handle.criterias.push(RelationCriteria::ById { field1: field1_, field2: field2_, comparison: comparison_ });
                    }
                }
            } 
            table_storage_path.components.pop();
            table_storage_path.components.pop();
        }
    }

    pub fn get_fields(&mut self, schema: &mut Schema) -> Result<(), String> {
        let mut template_path = HBAMPath::new(vec!["", "3", "5", ""]);
        for table_id in schema.tables.clone().keys() {
            template_path.components[0] = table_id.to_string();
            for field_id in 1..255 {
                template_path.components[3] = field_id.to_string();
                if let Ok(()) = self.goto_directory(&template_path) {
                    let name_ = fm_string_decrypt(&self.get_kv_value(16).expect("Unable to get KV."));
                    let table_handle = schema.tables.get_mut(table_id).expect("Corrupted table ID does not exist for field.");
                    table_handle.fields.insert(field_id, Field {
                        id: field_id,
                        name: name_,
                        created_by: String::new(),
                        modified_by: String::new(),
                        autoentry: AutoEntry {
                            definition: AutoEntryType::NA,
                            nomodify: false,
                        },
                        validation: Validation {
                            trigger: ValidationTrigger::OnEntry,
                            user_override: false,
                            checks: vec![],
                            message: String::from("Error with validation."),
                        },
                        global: false,
                        repetitions: 1,

                    });
                }
            }
        }
        Ok(())
    }

    pub fn get_layouts(&mut self, schema: &mut Schema) -> Result<(), String> {
        let mut template_path = HBAMPath::new(vec!["4", "1", "7", ""]);
        for layout_id in 1..255 {
            template_path.components[3] = layout_id.to_string();
            if let Ok(()) = self.goto_directory(&template_path) {
                let definition = self.get_kv_value(2).expect("unable to get layout definition.");
                let name_ = fm_string_decrypt(&self.get_kv_value(16).expect("Unable to find layout name in file."));
                schema.layouts.insert(layout_id, LayoutFM {
                    id: layout_id,
                    name: name_,
                    table_occurrence: definition[1] as usize,
                    table_occurrence_name: String::new(),
                });
            }
        }
        Ok(())
    }

    fn goto_directory(&mut self, path: &HBAMPath) -> Result<(), String> {
        let mut block = self.inner.get_leaf(path);
        loop {
            for offset in 0..block.chunks.len() {
                let chunk = block.chunks[offset].chunk();
                if chunk.path.components == *path.components {
                    self.inner.cursor = HBAMCursor { block_index: block.index, chunk_index: offset as u16 };
                    return Ok(())
                } else if chunk.path > *path || block.index == 0 {
                    return Err(format!("Directory {:?} not found.", path));
                }
            }
            let n = block.next;
            if n ==  0 {
                return Err("Unable to find directory.".to_string());
            }
            block = self.inner.load_leaf_n_from_disk(n).expect("Unable to get next leaf.");
        }
    }

    fn get_simple_data(&mut self) -> Result<Vec<Vec<u8>>, String> {
        let mut res = vec![];
        let mut start = self.inner.cursor.chunk_index;
        let dir_path = self.inner.get_current_block().chunks[self.inner.cursor.chunk_index as usize].chunk().path.clone();
        let (mut block, mut buffer) = self.inner.get_current_block_with_buffer_mut();
        loop {
            for offset in start as usize..block.chunks.len() {
                let wrapper = &block.chunks[offset];
                let chunk = wrapper.chunk();
                if chunk.ctype == InstructionType::DataSimple {
                    if dir_path == chunk.path {
                        res.push(chunk.data.unwrap().lookup_from_buffer(&buffer).expect("Unable to read simple data from buffer."));
                    }
                } else if chunk.path > dir_path {
                    return Ok(res);
                }
            }
            (block, buffer) = self.inner.get_next_leaf_with_buffer_mut().expect("Unable to get next leaf.");
            start = 0;
        }
    }

    fn get_kv(&mut self, key: u16) -> Result<ChunkType, String> {
        let mut block = self.inner.get_current_block();
        let mut start = self.inner.cursor.chunk_index;
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

    fn get_kv_value(&mut self, key: u16) -> Result<Vec<u8>, String> {
        let mut start = self.inner.cursor.chunk_index;
        let block_start = self.inner.cursor.block_index;
        let dir_path = self.inner.get_current_block().chunks[self.inner.cursor.chunk_index as usize].chunk().path.clone();
        let (mut block, mut buffer) = self.inner.get_current_block_with_buffer();
        loop {
            for offset in start as usize..block.chunks.len() {
                let wrapper = &block.chunks[offset];
                let chunk = wrapper.chunk();
                if chunk.ref_simple == Some(key) {
                    if dir_path.components == chunk.path.components {
                        let storage = match wrapper {
                            ChunkType::Modification(..) => &self.staging_buffer.buffer,
                            ChunkType::Unchanged(..) => &buffer,
                        };
                        return Ok(chunk.data.unwrap().lookup_from_buffer(storage).expect("Unable to lookup data from buffer."));
                    }
                } else if chunk.path > dir_path {
                    return Err(format!("Key {} not found in directory {:?}", key, dir_path));
                }
            }
            (block, buffer) = self.inner.get_next_leaf_with_buffer().expect("Unable to get next leaf.");
            start = 0;
        }
    }

    fn set_kv(&mut self, key: u16, data: &[u8]) -> Result<(), String> {
        let mut start = self.inner.cursor.chunk_index;
        let dir_path = self.inner.get_current_block().chunks[self.inner.cursor.chunk_index as usize].chunk().path.clone();
        let mut block = self.inner.get_current_block_mut();
        loop {
            for offset in start as usize..block.chunks.len() {
                let wrapper = &mut (&mut block.chunks[offset]);
                let chunk = wrapper.chunk_mut();
                if chunk.ref_simple == Some(key) {
                    if dir_path == chunk.path {
                        chunk.data = Some(self.staging_buffer.store(data.to_vec()));
                        **wrapper = ChunkType::Modification(chunk.clone());
                        return Ok(())
                    }
                } else if chunk.path > dir_path {
                    return Err(format!("Key {} not found in directory {:?}", key, dir_path));
                }
            }
            block = self.inner.get_next_leaf_mut().expect("Unable to get next leaf.");
            start = 0;
        }
    }

    fn set_long_kv(&mut self, key: &Vec<u8>, data: &[u8]) -> Result<(), String> {
        let mut start = self.inner.cursor.chunk_index;
        let dir_path = self.inner.get_current_block().chunks[self.inner.cursor.chunk_index as usize].chunk().path.clone();
        let (mut block, mut buffer) = self.inner.get_current_block_with_buffer_mut();
        loop {
            for offset in start as usize..block.chunks.len() {
                let wrapper = &mut (&mut block.chunks[offset]);
                if wrapper.chunk().ref_data.is_none() { continue; }
                let storage = match wrapper {
                    ChunkType::Modification(..) => &self.staging_buffer.buffer,
                    ChunkType::Unchanged(..) => &buffer,
                };
                let chunk = &mut wrapper.chunk_mut();
                if let Ok(key) = chunk.ref_data.unwrap().lookup_from_buffer(storage) {
                    if dir_path == chunk.path {
                        let key_location = self.staging_buffer.store(key.to_vec());
                        chunk.ref_data = Some(key_location);
                        let data_location = self.staging_buffer.store(data.to_vec());
                        chunk.data = Some(data_location);
                        **wrapper = ChunkType::Modification(chunk.clone());
                        return Ok(())
                    }
                } else if chunk.path > dir_path {
                    return Err(format!("Key {:?} not found in directory {:?}", key, dir_path));
                }
            }
            self.inner.get_next_leaf_mut().expect("Unable to get next leaf.");
            (block, buffer) = self.inner.get_current_block_with_buffer_mut();
            start = 0;
        }
    }

    fn set_long_kv_by_data(&mut self, key: &Vec<u8>, data: &[u8]) -> Result<(), String> {
        let mut start = self.inner.cursor.chunk_index;
        let dir_path = self.inner.get_current_block().chunks[self.inner.cursor.chunk_index as usize].chunk().path.clone();
        let (mut block, mut buffer) = self.inner.get_current_block_with_buffer_mut();
        loop {
            for offset in start as usize..block.chunks.len() {
                let wrapper = &mut block.chunks[offset];
                if wrapper.chunk().data.is_none() || wrapper.chunk().ref_data.is_none() { continue; }
                let storage = match wrapper {
                    ChunkType::Modification(..) => &self.staging_buffer.buffer,
                    ChunkType::Unchanged(..) => &buffer,
                };
                let chunk = wrapper.chunk_mut();
                if let Ok(chunk_data) = chunk.data.unwrap().lookup_from_buffer(storage) {
                    if dir_path == chunk.path {
                        let key_location = self.staging_buffer.store(key.to_vec());
                        chunk.ref_data = Some(key_location);
                        let data_location = self.staging_buffer.store(chunk_data);
                        chunk.data = Some(data_location);
                        let new = chunk.clone();
                        *wrapper = ChunkType::Modification(new);
                        return Ok(())
                    }
                } else if chunk.path > dir_path {
                    return Err(format!("Key {:?} not found in directory {:?}", key, dir_path));
                }
            }
            self.inner.get_next_leaf().unwrap();
            (block, buffer) = self.inner.get_current_block_with_buffer_mut();
            start = 0;
        }
    }

    fn get_long_kv(&mut self, key: &Vec<u8>) -> Result<ChunkType, String> {
        let mut start = self.inner.cursor.chunk_index;
        let dir_path = self.inner.get_current_block().chunks[self.inner.cursor.chunk_index as usize].chunk().path.clone();
        let (mut block, buffer) = self.inner.get_current_block_with_buffer();
        loop {
            for offset in start as usize..block.chunks.len() {
                let wrapper = &block.chunks[offset];
                if wrapper.chunk().ref_data.is_none() { continue; }
                let storage = match wrapper {
                    ChunkType::Modification(..) => &self.staging_buffer.buffer,
                    ChunkType::Unchanged(..) => &buffer,
                };
                let chunk = wrapper.chunk();
                if let Ok(key) = chunk.ref_data.unwrap().lookup_from_buffer(storage) {
                    if dir_path == chunk.path {
                        return Ok(wrapper.clone());
                    }
                } else if chunk.path > dir_path {
                    return Err(format!("Key {:?} not found in directory {:?}", key, dir_path));
                }
            }
            block = self.inner.get_next_leaf_mut().expect("Unable to get next leaf.");
            start = 0;
        }
    }

    fn update_consistency_counter(&mut self) -> Result<(), String> {
        let mut start = self.inner.cursor.chunk_index;
        let dir_path = self.inner.get_current_block().chunks[self.inner.cursor.chunk_index as usize].chunk().path.clone();
        let (mut block, mut buffer) = self.inner.get_current_block_with_buffer_mut();
        loop {
            for offset in start as usize..block.chunks.len() {
                let wrapper = &mut (&mut block.chunks[offset]);
                let storage = match wrapper {
                    ChunkType::Modification(..) => &self.staging_buffer.buffer,
                    ChunkType::Unchanged(..) => &buffer,
                };
                let chunk = wrapper.chunk_mut();
                if chunk.ref_simple == Some(252) {
                    if dir_path.components == chunk.path.components {
                        let n_buffer = chunk.data.unwrap().lookup_from_buffer(storage).expect("Unable to retrieve data from storage.");

                        let n = get_int(&n_buffer[1..=n_buffer[0] as usize]) + 1;
                        let mut n_bytes = put_path_int(n as u32);
                        n_bytes.insert(0, n_bytes.len() as u8);
                        chunk.data = Some(self.staging_buffer.store(n_bytes));
                        **wrapper = ChunkType::Modification(chunk.clone());
                        return Ok(())
                    }
                } else if chunk.path > dir_path {
                    return Err(format!("Key {} not found in directory {:?}", 252, dir_path));
                }
            }
            self.inner.get_next_leaf_mut().expect("Unable to get next leaf.");
            (block, buffer) = self.inner.get_current_block_with_buffer_mut();
            start = 0;
        }
    }

    pub fn modify_table(&mut self, table: &Table) {
        self.goto_directory(&HBAMPath::new(vec!["3", "16", "1", "1"])).expect("Unable to go to directory.");
        let mut id_data = put_int(table.id);
        id_data.insert(0, id_data.len() as u8);
        let mut de_name = encode_text(&table.name);
        de_name.append(&mut vec![0; 3]);
        self.set_long_kv_by_data(&de_name, &id_data).expect("Unable to set long kv using data.");

        self.goto_directory(&HBAMPath::new(vec!["3", "16", "1"])).expect("Unable to go to directory.");
        self.update_consistency_counter().expect("Unable to increment keyvalue.");

        self.goto_directory(&HBAMPath::new(vec!["3", "16", "5", &table.id.to_string()])).expect("Unable to go to directory.");
        self.set_kv(16, &fm_string_encrypt(&table.name)).expect("Unable to set keyvalue pair.");
        self.update_consistency_counter().expect("Unable to increment keyvalue.");
        
        self.goto_directory(&HBAMPath::new(vec!["3", "17", "1"])).expect("Unable to go to directory.");
        self.update_consistency_counter().expect("Unable to increment keyvalue.");

        self.goto_directory(&HBAMPath::new(vec!["4", "1"])).expect("Unable to go to directory.");
        self.update_consistency_counter().expect("Unable to increment keyvalue.");

        self.goto_directory(&HBAMPath::new(vec!["4", "5", "1"])).expect("Unable to go to directory.");
        self.update_consistency_counter().expect("Unable to increment keyvalue.");

        self.goto_directory(&HBAMPath::new(vec!["2"])).expect("Unable to get to directory.");
        let mut change_data = self.get_kv_value(8).expect("Unable to get keyvalue");
        change_data[42] += 1;
        change_data[157] += 1;
        self.set_kv(8, &change_data).expect("Unable to set keyvalue.");

        let mut change_data = self.get_kv_value(9).expect("Unable to get keyvalue");
        change_data[36] += 1;
        self.set_kv(9, &change_data).expect("Unable to set keyvalue.");
    }

    pub fn modify_table_occurrence(&mut self, table_occurrence: &TableOccurrence) {
        self.goto_directory(&HBAMPath::new(vec!["3", "17", "5", &table_occurrence.id.to_string()])).expect("Unable to go to table occurrence storage directory.");
        self.set_kv(16, &fm_string_encrypt(&table_occurrence.name)).expect("Unable to set kv.");
        self.update_consistency_counter().expect("Unable to update consistency counter");
    }

    pub fn commit_table_changes(&mut self, schema: &Schema, diffs: &DiffCollection) {
        for table in &schema.tables {
            let status = diffs.get(&table.1.id).expect("diff does not include information for table.");
            match status {
                DBObjectStatus::Unmodified => {},
                DBObjectStatus::Modified => {
                    self.modify_table(table.1);
                }
                DBObjectStatus::Created => {},
                DBObjectStatus::Deleted => {},
            }
        }
        self.inner.write_nodes(&self.staging_buffer).expect("Unable to write nodes to output file.");
    }

    pub fn commit_table_occurrence_changes(&mut self, schema: &Schema, diffs: &DiffCollection) {
        for table_occurrence in &schema.table_occurrences {
            let status = diffs.get(&table_occurrence.1.id).expect("diff does not include information for table.");
            match status {
                DBObjectStatus::Unmodified => {},
                DBObjectStatus::Modified => {
                    self.modify_table_occurrence(table_occurrence.1);
                }
                DBObjectStatus::Created => {},
                DBObjectStatus::Deleted => {},
            }
        }
        self.inner.write_nodes(&self.staging_buffer).expect("Unable to write nodes to output file.");
    }

    pub fn commit_changes(&mut self, schema: &Schema, diffs: &SchemaDiff) {
        self.commit_table_changes(schema, &diffs.tables);
        self.commit_table_occurrence_changes(schema, &diffs.table_occurrences);
        self.inner.write_nodes(&self.staging_buffer).expect("Unable to write table block to file.");
    }
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
        // 2 blocks in cache because of the root block.
        assert_eq!(file.inner.cached_blocks.len(), 2);
        assert!(file.inner.cached_blocks.contains_key(&64));

        file.goto_directory(&HBAMPath::new(vec!["6"])).expect("Unable to go to directory.");
        assert!(file.inner.cached_blocks.contains_key(&64)
            && file.inner.cached_blocks.contains_key(&62));
        // Fault line between block 62 and 61.
        file.goto_directory(&HBAMPath::new(vec!["6", "5", "1", "14", "0"])).expect("Unable to go to directory.");
        assert!(file.inner.cached_blocks.contains_key(&64)
            && file.inner.cached_blocks.contains_key(&62)
            && file.inner.cached_blocks.contains_key(&61));
        assert!(file.inner.cached_blocks.len() == 4);
    }

    #[test]
    fn kv_retrieval_test() {
        let mut file = HBAMInterface::new(Path::new("test_data/input/blank.fmp12"));
        file.goto_directory(&HBAMPath::new(vec!["3", "16", "5", "129"])).expect("Unable to go to directory.");
        let buffer = file.inner.get_buffer_from_leaf(file.inner.cursor.block_index as u64);

        let kv = file.get_kv(16).unwrap();
        assert_eq!(kv.chunk().data.unwrap().lookup_from_buffer(&buffer).unwrap(), vec![56, 54, 59, 52, 49]);

        let kv = file.get_kv(800);
        assert!(kv.is_err());   

        // Fault line between block 8 and 7
        file.goto_directory(&HBAMPath::new(vec!["23", "2", "1"])).expect("Unable to go to directory.");
        let kv = file.get_kv(0);
        assert!(kv.is_ok());
        let kv = file.get_kv(4).unwrap();
        let buffer = file.inner.get_buffer_from_leaf(file.inner.cursor.block_index as u64);
        assert_eq!(kv.chunk().data.unwrap().lookup_from_buffer(&buffer).unwrap(), vec![0, 0, 0, 3]);

        file.goto_directory(&HBAMPath::new(vec!["2"])).expect("Unable to go to directory");

        let data = file.get_kv_value(8).expect("Unable to get keyvalue");

        // assert_eq!(data, vec![78, 152, 78, 152, 78, 152, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 10, 40, 53,
        //     122, 104, 106, 116, 107, 116, 104, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 1, 8, 107, 122, 107, 110, 116, 108, 116, 107, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn kv_set_test() {
        let mut file = HBAMInterface::new(Path::new("test_data/input/blank.fmp12"));
        file.goto_directory(&HBAMPath::new(vec!["3", "16", "5", "129"])).expect("Unable to go to directory.");
        let buffer = file.inner.get_buffer_from_leaf(file.inner.cursor.block_index as u64);

        let kv = file.get_kv(16).unwrap();
        assert_eq!(kv.chunk().data.unwrap().lookup_from_buffer(&buffer).unwrap(), vec![56, 54, 59, 52, 49]);

        file.set_kv(16, &[23, 15, 72, 112, 49]).expect("Unable to set keyvalue");
        let kv = file.get_kv(16).unwrap();
        assert_eq!(kv.chunk().data.unwrap().lookup_from_buffer(&file.staging_buffer.buffer).unwrap(), vec![23, 15, 72, 112, 49]);

    }

    #[test]
    fn long_kv_set_test() {
        let mut file = HBAMInterface::new(Path::new("test_data/input/blank.fmp12"));
        file.goto_directory(&HBAMPath::new(vec!["3", "16", "1", "1"])).expect("Unable to go to directory.");
        let buffer = file.inner.get_buffer_from_leaf(file.inner.cursor.block_index as u64);

        let key = &mut vec![18, 37, 19, 48, 18, 15, 19, 109, 19, 30, 0, 0, 0];
        let kv = file.get_long_kv(&key).unwrap();
        assert_eq!(kv.chunk().data.unwrap().lookup_from_buffer(&buffer).unwrap(), vec![2, 128, 1]);

        file.set_long_kv(key, &[2, 128, 2]).expect("Unable to set keyvalue");
        let kv = file.get_long_kv(&key).unwrap();
        assert_eq!(kv.chunk().data.unwrap().lookup_from_buffer(&file.staging_buffer.buffer).unwrap(), vec![2, 128, 2]);
        file.set_long_kv(key, &[2, 128, 1]).expect("Unable to set keyvalue");
        let kv = file.get_long_kv(&key).unwrap();
        assert_eq!(kv.chunk().data.unwrap().lookup_from_buffer(&file.staging_buffer.buffer).unwrap(), vec![2, 128, 1]);

        let key = &mut vec![19, 48, 18, 15, 19, 109, 19, 30, 0, 0, 0];
        file.set_long_kv_by_data(key, &[2, 128, 1]).expect("Unable to set keyvalue.");
        let kv = file.get_long_kv(&key).unwrap();
        assert_eq!(kv.chunk().ref_data.unwrap().lookup_from_buffer(&file.staging_buffer.buffer).unwrap(), vec![19, 48, 18, 15, 19, 109, 19, 30, 0, 0, 0]);
    }
}





