use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct CommandLine {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Test {
        #[clap(long)]
        cadmus_file: Option<String>,
        #[clap(long, conflicts_with = "cadmus_file")]
        fmp_file: Option<String>,
        #[clap(long)]
        tests: Option<Vec<String>>,
    },
    Sync {
        #[clap(long)]
        cadmus_file: Option<String>,
        #[clap(long)]
        fmp_file: Option<String>,
    },
    Hbam {
        #[clap(long, required = true)]
        fmp_file: Option<String>,
        #[clap(long = "print-directory", action)]
        print_directory: Option<String>,
        #[clap(long = "print-root-block", action)]
        print_root_block: bool,
        #[clap(long = "print-all-blocks", action)]
        print_all_blocks: bool,
        #[clap(long = "json-out", action)]
        json_out: bool,
        #[clap(long, action)]
        page_check: bool,
    }
}

