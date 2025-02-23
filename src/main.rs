use std::{fs::read_to_string, path::Path};
use clap::Parser;
use cli::CommandLine;
use hbam2::{api::{self}, page_store::PageStore, path::HBAMPath, chunk::{LocalChunk, LocalChunkContents, Chunk}};
use shell::Shell;

mod cadlang;
mod util;
mod staging_buffer;
mod burn_script;
mod hbam2;
mod schema;
mod fm_script_engine;
mod dbobjects;
mod emulator3;
mod cli;
mod shell;

fn main() -> Result<(), std::io::Error>{
    let args = CommandLine::parse();
    match args.command {
        cli::Command::Shell { } => {
            let mut env = emulator3::Emulator::new();
            let mut shell = Shell::new(&mut env);
            shell.main_loop().unwrap()
        },
        cli::Command::Test { file, tests } => {
            let mut env = emulator3::Emulator::new();
            let mut stored_tests = vec![];
            let filename = if let Some(fmp_file_uw) = file.fmp12_file {
                let code = read_to_string(Path::new(&fmp_file_uw)).unwrap();
                stored_tests.extend(cadlang::compiler::compile_to_file(code).unwrap().tests);
                fmp_file_uw
            } else if let Some(cadmus_file_uw) = file.cadmus_file {
                let code = read_to_string(Path::new(&cadmus_file_uw)).unwrap();
                stored_tests.extend(cadlang::compiler::compile_to_file(code).unwrap().tests);
                cadmus_file_uw
            } else {
                String::new()
            };


            println!("{:?}", stored_tests);

            if let Some(tests) = tests {
                for test in tests {
                    let to_run = stored_tests
                        .iter()
                        .find(|search| search.name == test)
                        .unwrap();
                    env.run_test_with_file(&to_run.name, &filename);
                }
            } else {
                for test in &stored_tests {
                    env.run_test_with_file(&test.name, &filename);
                }
            }
        },
        cli::Command::Sync { cadmus_file, fmp_file } => todo!(),
        cli::Command::Hbam { fmp_file, print_dir, .. } => {
            let fmp_file_uw = fmp_file.unwrap();
            let mut cache = PageStore::new();

            if print_dir.is_some() {
                println!("Printing key: {}", print_dir.as_ref().unwrap());
                let mut search = &mut HBAMPath::from_csv(print_dir.unwrap().as_ref()).unwrap();
                println!("Key: {}", search);
                let view = crate::hbam2::bplustree::get_view_from_key(
                    search,
                    &mut cache,
                    fmp_file_uw.as_ref())
                    .unwrap()
                    .unwrap();

                for f in view.chunks {
                    match f.contents {
                        LocalChunkContents::Push { ref key } => {
                            search.components.push(key.to_vec());
                        }
                        LocalChunkContents::Pop => {
                            search.components.pop();
                        }
                        _ => {}
                    }
                    println!("{}: {}",search, f);
                }
            }
        }
    }

    Ok(())
}




