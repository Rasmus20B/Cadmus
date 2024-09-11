use std::path::Path;
use fm_core::file_repr::FmpFileView;
use hbam::btree::HBAMFile;

mod data_cache;
mod fm_core;
mod fm_io;
mod fm_format;
mod util;
mod hbam;

fn main() {

    let mut file = HBAMFile::new(Path::new("../fm_vc/databases/Quotes.fmp12"));
    let leaf = file.get_leaf(vec!["3".to_string(), "17".to_string()]);
    // let leaf =  file.get_leaf_n(453);

    println!("GOT BLOCK.");
    for chunk in leaf.chunks {
        println!("{}", chunk);
    }

}
