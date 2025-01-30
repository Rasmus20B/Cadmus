use std::{fs::read_to_string, path::Path};
use cadlang::compiler::compile_to_schema;
use clap::Parser;
use cli::CommandLine;
use hbam2::{api::{self}, page_store::PageStore, path::HBAMPath};
use schema::Schema;
use emulator::test::TestEnvironment;

mod cadlang;
mod util;
mod staging_buffer;
mod burn_script;
mod hbam;
mod hbam2;
mod schema;
mod fm_script_engine;
mod dbobjects;
mod emulator;
mod emulator2;
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
    let mut ctx = InputContext::new();
    let args = CommandLine::parse();
    match args.command {
        cli::Command::Test { file, tests } => {
            if let Some(fmp_file_uw) = file.fmp12_file {
                ctx.fmp = Schema::new();
                let mut storage = PageStore::new();
                ctx.fmp.tables = api::get_table_catalog(&mut storage, &fmp_file_uw);
                let (occurrences, relations) = api::get_occurrence_catalog(&mut storage, &fmp_file_uw);
                ctx.fmp.table_occurrences = occurrences;
                ctx.fmp.relations = relations;
                ctx.fmp.layouts = api::get_layout_catalog(&mut storage, &fmp_file_uw);
            } else if let Some(cadmus_file_uw) = file.cadmus_file {
                ctx.fmp = match compile_to_schema(read_to_string(Path::new(&cadmus_file_uw))?) {
                    Err(e) => panic!("{}", e),
                    Ok(schema) => schema
                };
            } 

            if let Some(test_list) = tests {
                for test_file in test_list {
                    let tests = match compile_to_schema(read_to_string(Path::new(&test_file))?) {
                        Err(e) => panic!("{}", e),
                        Ok(schema) => schema
                    };
                    ctx.fmp.tests.extend(&mut tests.tests.into_iter());
                }
            }

            let mut te : TestEnvironment = TestEnvironment::new(&ctx.fmp);
            te.generate_test_environment();
            te.run_tests();
        },
        cli::Command::Sync { cadmus_file, fmp_file } => todo!(),
        cli::Command::Hbam { mut fmp_file, print_dir, .. } => {
            let fmp_file_uw = fmp_file.unwrap();
            let mut cache = PageStore::new();

            if print_dir.is_some() {
                println!("Printing key: {}", print_dir.as_ref().unwrap());
                let key = &HBAMPath::from_csv(print_dir.unwrap().as_ref()).unwrap();
                println!("Key: {}", key);
                let view = crate::hbam2::bplustree::get_view_from_key(
                    key,
                    &mut cache,
                    fmp_file_uw.as_ref())
                    .unwrap()
                    .unwrap();

                for f in view.chunks {
                    println!("{:?}", f);
                }
            }
        }
    }

    Ok(())
}




