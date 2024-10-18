use std::{fs::{File, OpenOptions}, io::{BufReader, BufWriter}, path::Path};
use clap::Parser;
use cli::CommandLine;
use diff::get_diffs;
use hbam::{chunk::Chunk, fs::HBAMInterface, path::HBAMPath};
use schema::Schema;


mod cadlang;
mod util;
mod staging_buffer;
mod compile;
mod burn_script;
mod hbam;
mod schema;
mod fm_script_engine;
mod emulator;
mod cli;
mod diff;

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
    let base_file: Option<HBAMInterface>;
    let mut ctx = InputContext::new();

    let args = CommandLine::parse();
    if args.input.is_some() {
        let in_file = File::open(Path::new(&args.input.clone().unwrap()))?;
        let reader = BufReader::new(in_file);
        let objects: Schema = serde_json::from_reader(reader).expect("Unable to read text input file");
        ctx.cad.tables.extend(objects.tables);
        ctx.cad.table_occurrences.extend(objects.table_occurrences);
    }

    if args.fmp.is_some() {
        base_file = Some(HBAMInterface::new(Path::new(&args.fmp.as_ref().unwrap())));
        let mut base_file = base_file.unwrap();
        ctx.fmp = Schema::from(&mut base_file);

        if args.print_directory.is_some() {
            let dir = HBAMPath::from(args.print_directory.unwrap());
            let (leaf, buffer) = base_file.inner.get_leaf_with_buffer(&dir);
            for wrapper in leaf.chunks {
                let chunk = Chunk::from(wrapper);
                println!("{}", chunk.chunk_to_string(&buffer))
            }
        }

        if args.print_all_blocks {
            base_file.inner.print_all_chunks();
        }

        if args.json_out {
            let path_text = args.fmp.clone().unwrap();
            let path = Path::new(&path_text);
            let json_path = path.with_file_name(path.file_name().unwrap().to_str().unwrap().strip_suffix(".fmp12").unwrap().to_string() + ".json");
            let writer = BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
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

        if args.test {
            if let Some(test_file) = args.test_file {
                let in_file = File::open(Path::new(&test_file))?;
                let reader = BufReader::new(in_file);
                let objects: Schema = serde_json::from_reader(reader).expect("Unable to read text input file");
                ctx.cad.tests.extend(objects.tests);
            } else {
            }
        }
    } else if args.input.is_some() {
        /* Generate a clean file based on schema. */
    }
    Ok(())
}




