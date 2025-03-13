mod cli;
mod error;
mod commands;
use clap::Parser;

use cli::CommandLine;
use common::hbam2::get_schema_contents;
use commands::init_cadmus_repo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CommandLine::parse();
    match args.command {
        cli::Command::Init => {
            init_cadmus_repo(&std::env::current_dir()?)?
        }
        _ => {}
    }

    Ok(())
}
