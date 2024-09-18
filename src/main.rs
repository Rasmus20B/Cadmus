use std::{fs::{write, File}, io::{BufRead, BufReader, Error}, path::Path};
use clap::Parser;
use cli::CLI;
use diff::{get_diff, DiffCollection};
use fm_core::file_repr::FmpFileView;
use fm_format::chunk::{Chunk, InstructionType};
use fm_io::block::Block;
use hbam::{btree::HBAMFile, path::HBAMPath};
use schema::{DBObject, DBObjectKind, Schema};
use serde::{Deserialize, Serialize};
use util::encoding_util::fm_string_decrypt;

use crate::{fm_format::chunk::ChunkType, staging_buffer::DataStaging, util::{dbcharconv::{self, encode_text}, encoding_util::{fm_string_encrypt, get_int}}};

mod data_cache;
mod fm_core;
mod fm_io;
mod fm_format;
mod util;
mod staging_buffer;
mod hbam;
mod schema;
mod cli;
mod diff;


fn load_tables(file: &mut HBAMFile) -> Vec<DBObject> {
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
                        tables.push(DBObject { id: x.parse().unwrap(), name: decoded, kind: DBObjectKind::Table });
                    }
                }
            }
            _ => {}
        };

    }
    tables
}

struct InputContext {
    cad: Schema,
    fmp: Schema,
}

impl InputContext {
    pub fn new() -> Self {
        Self {
            cad: Schema::new(),
            fmp: Schema::new(),
        }
    }
}


fn main() -> Result<(), std::io::Error>{
    let mut file: Option<HBAMFile> = None;
    let mut ctx = InputContext::new();

    let args = CLI::parse();
    if args.input.is_some() {
        let in_file = File::open(Path::new(&args.input.as_ref().unwrap()))?;
        let reader = BufReader::new(in_file);
        ctx.cad.objects.extend(serde_json::from_reader(reader));
    }

    if args.fmp.is_some() {
        file = Some(HBAMFile::new(Path::new(&args.fmp.as_ref().unwrap())));
        ctx.fmp.objects.extend(load_tables(&mut file.as_mut().unwrap()));
    }

    if args.sync {
        let diffs = get_diff(&ctx.fmp, &ctx.cad);
        file.unwrap().commit_changes(&diffs);
    } else if args.input.is_some() {
        /* Generate a clean file based on schema. */
    }
    Ok(())
}




