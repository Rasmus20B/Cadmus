use clap::Parser;

#[derive(Parser)]
#[command(arg_required_else_help(true))]
pub struct CLI {
    #[clap(short = 'i')]
    pub input: Option<String>,
    #[clap(long = "fmp")]
    pub fmp: Option<String>,
    #[clap(long = "print-directory", requires("fmp"))]
    pub print_directory: Option<String>,
    #[clap(long = "print-root-block", action, requires("fmp"))]
    pub print_root_block: bool,
    #[clap(short = 's', requires("fmp"), requires("input"))]
    pub sync: bool,
    #[clap(long = "print-all-blocks", action, requires("fmp"))]
    pub print_all_blocks: bool,
}

