use std::path::Path;
use fm_core::file_repr::FmpFileView;
use hbam::{btree::HBAMFile, path::HBAMPath};

mod data_cache;
mod fm_core;
mod fm_io;
mod fm_format;
mod util;
mod hbam;

fn main() {

    let mut file = HBAMFile::new(Path::new("../fm_vc/databases/Quotes.fmp12"));
    let hbampath = HBAMPath::new(vec!["3", "17"]);
    let leaf = file.get_leaf(&hbampath);

    for chunk in leaf.chunks {
        println!("{}", chunk);
    }

}
