
use std::{collections::BTreeMap, iter::zip};

use crate::{dbobjects::file::File, hbam2::chunk::{Chunk, LocalChunk, LocalChunkContents}, util::encoding_util::put_path_int};

use super::{super::HBAMPath, error::Result};

pub trait DataTree {
    fn insert(&mut self, path: HBAMPath, value: LocalChunkContents) -> Result<()>;
    fn flatten(&self) -> Result<Vec<LocalChunk>>;
}

#[derive(Clone, Debug)]
pub enum NodeValue {
    KeyValue { key: usize, value: Vec<u8> },
    Simple { value: Vec<u8> },
}

#[derive(Default, Debug)]
pub struct DataBTree {
    map: BTreeMap<HBAMPath, Vec<LocalChunkContents>>,
}

impl DataTree for DataBTree {
    fn insert(&mut self, path: HBAMPath, value: LocalChunkContents) -> Result<()> {
        self.map.entry(path).and_modify(|files| files.push(value.clone())).or_insert(vec![value]);
        Ok(())
    }

    fn flatten(&self) -> Result<Vec<LocalChunk>> {
        let mut current_path = HBAMPath::new(vec![]);
        for (path, entry) in &self.map {
            let common_prefix = path.components
                .iter()
                .zip(&current_path.components)
                .take_while(|(a, b)| a == b)
                .count();

            for _ in common_prefix..current_path.components.len() {
                println!("POP");
            }

            for dir in &path.components[common_prefix..] {
                println!("PUSH {:?}", dir);
            }

            for file in entry {
                match file {
                    LocalChunkContents::SimpleRef { key, data } => {
                        println!("kv {} := {:?}", key, data);
                    }
                    LocalChunkContents::SimpleData { data } => {
                        println!("data := {:?}", data);
                    }
                    _ => {},
                }
            }
            current_path = path.clone();
        }

        Ok(vec![])
    }
}

impl From<File> for DataBTree {
    fn from(value: File) -> Self {
        let mut result = DataBTree::default();

        result.insert(
            HBAMPath::new(vec![&[2]]), 
            LocalChunkContents::SimpleRef { key: 3, data: "20.1".as_bytes().to_vec() }
        )
        .unwrap();

        result.insert(HBAMPath::new(vec![&[3], &[16], &[1]]), LocalChunkContents::SimpleRef {
            key: 0,
            data: put_path_int(value.schema.tables.len() as u32)
        }).unwrap();

        for table in value.schema.tables {
            result.insert(HBAMPath::new(vec![&[3], &[16], &[5], &[table.id as u8]]), 
                LocalChunkContents::SimpleRef {
                    key: 16, data: table.name.into()
                }).unwrap();

            for field in &table.fields {
                result.insert(
                    HBAMPath::new(vec![&[table.id as u8 + 128], &[3], &[5], &[field.1.id as u8]]),
                    LocalChunkContents::SimpleRef { key: 16, data: field.1.name.clone().into() }).unwrap();
            }
        }

        let mut occurrence_count = 0;

        for occurrence in value.schema.relation_graph.nodes {
            occurrence_count += 1;
            result.insert(
                HBAMPath::new(vec![&[3], &[17], &[1], &[1]]),
                LocalChunkContents::SimpleData { 
                    data: (occurrence.name + &occurrence.id.to_string()).as_bytes().to_vec() 
                },
            ).unwrap();

            result.insert(
                HBAMPath::new(vec![&[3], &[17], &[1], &[3]]), 
                LocalChunkContents::SimpleData { 
                    data: (occurrence_count.to_string() + &occurrence.id.to_string()).as_bytes().to_vec()
                }
            ).unwrap();

            result.insert(
                HBAMPath::new(vec![&[3], &[17], &[5], &[occurrence.id as u8]]), 
                LocalChunkContents::SimpleRef { 
                    key: 2,
                    data: (
                        occurrence.base.data_source.to_string() 
                        + ":" +
                        &occurrence.base.table_id.to_string()).as_bytes().to_vec() 
                }
                ).unwrap();
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::Path};

    use crate::cadlang;

    use super::*;

    #[test]
    fn data_tree_test() {
        let file = cadlang::compiler::compile_to_file(Path::new("./test_data/cad_files/multi_file_solution/quotes.cad")).unwrap();
        let result = DataBTree::from(file);

        for chunk in result.flatten().unwrap() {
            println!("{:?}", chunk);
        }
    }
}
