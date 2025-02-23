use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct CommandLine {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Args, Debug)]
#[group(required = true ,multiple = false)]
pub struct TestFile {
    #[clap(long)]
    pub cadmus_file: Option<String>,
    #[clap(long)]
    pub fmp12_file: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Test {
        #[clap(flatten)]
        file: TestFile,
        #[clap(long)]
        tests: Option<Vec<String>>,
    },
    Shell {
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
        #[clap(long = "print-dir", action)]
        print_dir: Option<String>,
        #[clap(long = "json-out", action)]
        json_out: bool,
        #[clap(long, action)]
        page_check: bool,
    }
}

