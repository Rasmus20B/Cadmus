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

use crate::{staging_buffer::DataStaging, util::{dbcharconv::{self, encode_text}, encoding_util::{fm_string_encrypt, get_int}}};

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
        match chunk.path.components[..].iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice() {
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
    let mut base_file: Option<HBAMFile> = None;
    let mut ctx = InputContext::new();

    let args = CLI::parse();
    if args.input.is_some() {
        let in_file = File::open(Path::new(&args.input.clone().unwrap()))?;
        let reader = BufReader::new(in_file);
        let objects: Vec<DBObject> = serde_json::from_reader(reader).expect("Unable to read text input file");
        ctx.cad.objects.extend(objects);
    }

    if args.fmp.is_some() {
        base_file = Some(HBAMFile::new(Path::new(&args.fmp.as_ref().unwrap())));

        if args.print_directory.is_some() {
            let dir = HBAMPath::from(args.print_directory.unwrap());
            let (leaf, buffer) = base_file.as_mut().unwrap().get_leaf_with_buffer(&dir);
            for wrapper in leaf.chunks {
                let chunk = Chunk::from(wrapper);
                println!("{}", chunk.chunk_to_string(&buffer))
            }
        }

        if args.print_all_blocks {
            base_file.as_mut().unwrap().print_all_chunks();
        }
    }

    if args.sync {
        let path_text = args.fmp.clone().unwrap();
        let path = Path::new(&path_text);
        let copy_path = path.with_file_name(path.file_name().unwrap().to_str().unwrap().strip_suffix(".fmp12").unwrap().to_string() + "_patch.fmp12");
        std::fs::copy(path, &copy_path).expect("Unable to create patch file.");
        let mut patch_file = HBAMFile::new(Path::new(&copy_path));
        ctx.fmp.objects.extend(load_tables(&mut patch_file));
        let diffs = get_diff(&ctx.fmp, &ctx.cad);
        let mut data_buffer = DataStaging::new();
        patch_file.commit_changes(&diffs, &mut data_buffer);
    } else if args.input.is_some() {
        /* Generate a clean file based on schema. */
    }
    Ok(())
}




