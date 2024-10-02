use std::{fs::{File, OpenOptions}, io::{BufReader, BufWriter}, path::Path};
use clap::Parser;
use cli::CLI;
use diff::get_diffs;
use fm_format::chunk::Chunk;
use hbam::{btree::HBAMFile, fs::HBAMInterface, path::HBAMPath};
use schema::{Schema, Table, TableOccurrence};
use util::encoding_util::fm_string_decrypt;


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

fn load_tables(file: &mut HBAMFile) -> Vec<Table> {
    let (block, buffer) = file.get_leaf_with_buffer(&HBAMPath::new(vec!["3", "16"]));
    let mut tables = vec![];
    for chunk_wrapper in &block.chunks {
        let chunk = Chunk::from(chunk_wrapper.clone());
        match chunk.path.components[..].iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice() {
            ["3", "16", "5", x] => {
                if chunk.ref_simple == Some(16) {
                    let data_uw = chunk.data.unwrap();
                    let string = data_uw.lookup_from_buffer(&buffer.to_vec()).expect("Unable to lookup data from file.");
                    let decoded = fm_string_decrypt(&string);
                    tables.push(Table { id: x.parse().unwrap(), name: decoded, created_by: "Admin".to_string(), modified_by: "admin".to_string() });
                }
            },
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
    let mut base_file: Option<HBAMInterface>;
    let mut ctx = InputContext::new();

    let args = CLI::parse();
    if args.input.is_some() {
        let in_file = File::open(Path::new(&args.input.clone().unwrap()))?;
        let reader = BufReader::new(in_file);
        let objects: Schema = serde_json::from_reader(reader).expect("Unable to read text input file");
        ctx.cad.tables.extend(objects.tables);
        ctx.cad.table_occurrences.extend(objects.table_occurrences);
    }

    if args.fmp.is_some() {
        base_file = Some(HBAMInterface::new(Path::new(&args.fmp.as_ref().unwrap())));
        let additions = base_file.as_mut().unwrap().get_tables();
        ctx.fmp.tables.extend(additions);
        let additions = base_file.as_mut().unwrap().get_table_occurrences();
        ctx.fmp.table_occurrences.extend(additions);
        let additions = base_file.as_mut().unwrap().get_relations();
        ctx.fmp.relations.extend(additions);

        if args.print_directory.is_some() {
            let dir = HBAMPath::from(args.print_directory.unwrap());
            let (leaf, buffer) = base_file.as_mut().unwrap().inner.get_leaf_with_buffer(&dir);
            for wrapper in leaf.chunks {
                let chunk = Chunk::from(wrapper);
                println!("{}", chunk.chunk_to_string(&buffer))
            }
        }

        if args.print_all_blocks {
            base_file.as_mut().unwrap().inner.print_all_chunks();
        }

        if args.json_out {
            let path_text = args.fmp.clone().unwrap();
            let path = Path::new(&path_text);
            let json_path = path.with_file_name(path.file_name().unwrap().to_str().unwrap().strip_suffix(".fmp12").unwrap().to_string() + ".json");
            let writer = BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(json_path)
                    .expect("Unable to open file."));
            serde_json::to_writer_pretty(writer, &ctx.fmp).expect("Unable to write JSON output.");
        }
    }

    if args.sync {
        let path_text = args.fmp.clone().unwrap();
        let path = Path::new(&path_text);
        let copy_path = path.with_file_name(path.file_name().unwrap().to_str().unwrap().strip_suffix(".fmp12").unwrap().to_string() + "_patch.fmp12");
        std::fs::copy(path, &copy_path).expect("Unable to create patch file.");
        let mut patch_file = HBAMInterface::new(Path::new(&copy_path));
        let additions = patch_file.get_tables();
        ctx.fmp.tables.extend(additions);
        let table_diffs = get_diffs(&ctx.fmp, &ctx.cad);
        patch_file.commit_changes(&ctx.cad, &table_diffs);
    } else if args.input.is_some() {
        /* Generate a clean file based on schema. */
    }
    Ok(())
}




