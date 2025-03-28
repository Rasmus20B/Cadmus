use std::{fs::read_to_string, path::Path};
use crate::shell::Host;
use clap::Parser;
use cli::CommandLine;
use hbam2::{api::{self}, page_store::PageStore, path::HBAMPath, chunk::{LocalChunk, LocalChunkContents, Chunk}};
use shell::Shell;

mod cadlang;
mod util;
mod hbam2;
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
            let filename = if let Some(fmp_file_uw) = file.fmp12_file {
                fmp_file_uw
            } else if let Some(cadmus_file_uw) = file.cadmus_file {
                cadmus_file_uw
            } else {
                String::new()
            };
            println!("file: {}", filename);
            env.run_tests(&filename);
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




