use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct CommandLine {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init,
    Add {
        #[clap(required = true)]
        files: Option<Vec<String>>,
    },
    Commit {
        message: Option<String>,
    },
    Pull {
    },
    Push {
        #[clap(long)]
        cadmus_file: Option<String>,
        #[clap(long)]
        fmp_file: Option<String>,
    },
    Clone {
        #[clap(required = true)]
        url: Option<String>,
    }
}


