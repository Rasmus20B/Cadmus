use std::{fs::File, io::{BufRead, BufReader, Error}, path::Path};
use fm_core::file_repr::FmpFileView;
use fm_format::chunk::{Chunk, InstructionType};
use fm_io::block::Block;
use hbam::{btree::HBAMFile, path::HBAMPath};
use serde::{Deserialize, Serialize};
use util::encoding_util::fm_string_decrypt;

use crate::{fm_format::chunk::ChunkType, staging_buffer::DataStaging, util::encoding_util::fm_string_encrypt};

mod data_cache;
mod fm_core;
mod fm_io;
mod fm_format;
mod util;
mod staging_buffer;
mod hbam;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
struct Table {
    pub id: usize,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
enum TableKind {
    Created(Table),
    Modified(Table),
    UnModified(Table),
    Deleted(Table),
}

impl From<TableKind> for Table {
    fn from(value: TableKind) -> Self {
        match value {
            TableKind::Created(table) 
                | TableKind::Deleted(table) 
                | TableKind::Modified(table) 
                | TableKind::UnModified(table) => {
                    table
            }
        }
    }
}


fn load_tables(file: &mut HBAMFile) -> Vec<Table> {
    let (block, buffer) = file.get_leaf_with_buffer(&HBAMPath::new(vec!["3", "16"]));
    let mut tables = vec![];
    for chunk_wrapper in &block.chunks {
        let chunk = Chunk::from(chunk_wrapper.clone());
        match chunk.path[..].iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice() {
            ["3", "16", "5", x] => {
                if chunk.ref_simple.is_some() {
                    if chunk.ref_simple.unwrap() == 16 {
                        let data_uw = chunk.data.unwrap();
                        let string = data_uw.lookup_from_buffer(&buffer.to_vec()).expect("Unable to lookup data from file.");
                        let decoded = fm_string_decrypt(&string);
                        tables.push(Table { id: x.parse().unwrap(), name: decoded });
                    }
                }
            }
            _ => {}
        };

    }
    tables
}

fn tables_diff(input: &Vec<Table>, original: &Vec<Table>) -> Vec<TableKind> {

    let mut created = input.iter()
        .filter(|in_table| !original.iter()
            .map(|table| table.id).collect::<Vec<_>>()
            .contains(&in_table.id))
        .map(|table| TableKind::Created(table.clone()))
        .collect::<Vec<_>>();

    let mut modified = input.iter()
        .filter(|table| original.iter().map(|table| table.id).collect::<Vec<_>>().contains(&table.id))
        .filter(|table| !original.iter().map(|table| table.name.clone()).collect::<Vec<_>>().contains(&table.name))
        .map(|table| TableKind::Modified(table.clone()))
        .collect::<Vec<_>>();

    let mut unmodified = input.iter()
        .filter(|table| original.contains(table))
        .map(|table| TableKind::UnModified(table.clone()))
        .collect::<Vec<_>>();

    let mut deleted = original.iter()
        .filter(|table| !input
            .iter()
            .map(|new_table| new_table.id).collect::<Vec<_>>()
            .contains(&table.id))
        .map(|table| TableKind::Deleted(table.clone())).collect::<Vec<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let mut result = Vec::<TableKind>::new();
    result.append(&mut created);
    result.append(&mut modified);
    result.append(&mut unmodified);
    result.append(&mut deleted);

    result.sort_by(|table1, table2| Table::from(table1.clone()).id.cmp(&Table::from(table2.clone()).id));
    result
}

fn main() -> Result<(), std::io::Error>{
    let mut file = HBAMFile::new(Path::new("test_data/input/blank.fmp12"));
    let hbampath = HBAMPath::new(vec!["3", "17"]);
    let mut leaf = file.get_leaf(&hbampath);

    let mut old_tables = load_tables(&mut file);
    old_tables.sort_by(|a, b| a.id.cmp(&b.id));


    let json_out = serde_json::to_string_pretty(&old_tables).expect("Unable to generate json file");
    std::fs::write("inspect_output/test_decompile", json_out).expect("Unable to write to file.");

    let in_file = File::open(Path::new("inspect_output/test_input"))?;
    let reader = BufReader::new(in_file);

    let json_in: Vec<Table> = serde_json::from_reader(reader)?;

    let new_tables = tables_diff(&json_in, &old_tables);

    let mut tmp = DataStaging::new();
    for chunk in leaf.chunks.iter().map(|chunk_wrapper| Chunk::from(chunk_wrapper.clone())) {
        println!("{}", chunk);
    }
    for table in &new_tables {
        println!("looking for table: {}", Table::from(table.clone()).id);
        match table {
            TableKind::Modified(table) => {
                let location = tmp.store(fm_string_encrypt(table.name.clone()));
                let mut chunk_copy = leaf.chunks.iter()
                    .map(|chunk_wrapper| Chunk::from(chunk_wrapper.clone()))
                    .enumerate()
                    .filter(|(i, chunk)| {
                        chunk.ref_simple.is_some_and(|chunk| chunk == 16) 
                            && 
                        chunk.path == (&["3".to_string(), "16".to_string(), "5".to_string(), table.id.to_string()])})
                    .collect::<Vec<_>>()[0].clone();
                chunk_copy.1.data = Some(location);
                println!("FOUND IT");
                chunk_copy.1.opcode = match location.length {
                    1..=5 => location.length,
                    _ => 6,
                };
                let old_data_offset = Chunk::from(leaf.chunks[chunk_copy.0].clone()).data.unwrap();
                // println!("modified chunk: OLD: {:?}, NEW: {:?}", &leaf_buffer[old_data_offset.offset as usize..(old_data_offset.offset+old_data_offset.length) as usize], tmp.load(location));
                // println!("CHUNK OLD: {}\nCHUNK NEW: {}", Chunk::from(leaf.chunks[chunk_copy.0].clone()), Chunk::from(ChunkType::Modification(chunk_copy.1.clone())));
                leaf.chunks[chunk_copy.0] = ChunkType::Modification(chunk_copy.1);
            }
            _ => {}
        }
    }
    file.write_node(&leaf, &tmp).expect("Unable to write new chunks to block.");

    let mut new_file = HBAMFile::new(Path::new("test_data/input/blank.fmp12"));
    let mut old_tables = load_tables(&mut new_file);
    old_tables.sort_by(|a, b| a.id.cmp(&b.id));


    let json_out = serde_json::to_string_pretty(&old_tables).expect("Unable to generate json file");
    std::fs::write("inspect_output/test_decompile_patched", json_out).expect("Unable to write to file.");
    Ok(())
}
