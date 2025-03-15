use std::{fs::read_to_string, path::Path};
use crate::shell::Host;
use clap::Parser;
use cli::CommandLine;
use common::hbam2::Context;
use hbam2::{page_store::PageStore, path::HBAMPath, chunk::{LocalChunk, LocalChunkContents, Chunk}};
use shell::Shell;

mod cadlang;
mod util;
mod hbam2;
mod dbobjects;
mod cli;
mod shell;

fn main() -> Result<(), std::io::Error>{
    let args = CommandLine::parse();
    match args.command {
        cli::Command::Shell { } => {
        },
        cli::Command::Test { file, tests } => {
        },
        cli::Command::Sync { cadmus_file, fmp_file } => todo!(),
        cli::Command::Hbam { fmp_file, print_dir, print_all_blocks, .. } => {
            let fmp_file_uw = fmp_file.unwrap();
            let mut ctx = Context::new();

            if print_all_blocks {
                ctx.print_full(&fmp_file_uw)
            } else if print_dir.is_some() {
                //println!("Printing key: {}", print_dir.as_ref().unwrap());
                //let mut search = &mut HBAMPath::from_csv(print_dir.unwrap().as_ref()).unwrap();
                //println!("Key: {}", search);
                //let view = hbam2::bplustree::get_view_from_key(
                //    search,
                //    &mut ctx.cache,
                //    fmp_file_uw.as_ref())
                //    .unwrap()
                //    .unwrap();
                //
                //for f in view.chunks {
                //    match f.contents {
                //        LocalChunkContents::Push { ref key } => {
                //            search.components.push(key.to_vec());
                //        }
                //        LocalChunkContents::Pop => {
                //            search.components.pop();
                //        }
                //        _ => {}
                //    }
                //    println!("{}: {}",search, f);
                //}
            }
        }
    }

    Ok(())
}




