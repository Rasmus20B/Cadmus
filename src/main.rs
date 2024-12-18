use std::{fs::{read_to_string, File, OpenOptions}, io::{BufReader, BufWriter}, path::Path};
use cadlang::compiler::compile_to_schema;
use clap::Parser;
use cli::CommandLine;
use diff::get_diffs;
use hbam::fs::HBAMInterface;
use hbam2::{api::{self}, page_store::PageStore};
use schema::Schema;

mod cadlang;
mod util;
mod staging_buffer;
mod burn_script;
mod hbam;
mod hbam2;
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
    if let Some(ref input) = args.input {
        let code = read_to_string(Path::new(&input))?;
        let schema = match compile_to_schema(code) {
            Err(e) => panic!("{}", e),
            Ok(schema) => schema
        };
        
    }

    if args.fmp.is_some() {

        ctx.fmp = Schema::new();
        let storage = PageStore::new();

        if args.print_all_blocks {
            api::emit_file(args.fmp.as_ref().unwrap());
        }

        // if args.print_root_block {
        //     base_file.inner.print_root_block();
        // }

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
            } 
        }
    } else if args.input.is_some() {
        /* Generate a clean file based on schema. */
    }
    Ok(())
}




