use std::path::Path;
use fm_core::file_repr::FmpFileView;
use fm_format::chunk::{Chunk, InstructionType};
use fm_io::block::Block;
use hbam::{btree::HBAMFile, path::HBAMPath};
use util::encoding_util::fm_string_decrypt;

mod data_cache;
mod fm_core;
mod fm_io;
mod fm_format;
mod util;
mod hbam;

fn load_tables(block: &Block, buffer: &[u8]) -> Vec<String> {

    let mut res = vec![];
    for chunk_wrapper in &block.chunks {
        let chunk = Chunk::from(chunk_wrapper.clone());
        match chunk.path[..].iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice() {
            ["3", "16", "5", x] => {
                if chunk.ref_simple.is_some() {
                    if chunk.ref_simple.unwrap() == 16 {
                        let data_uw = chunk.data.unwrap();
                        let string = data_uw.lookup_from_buffer(&buffer.to_vec()).expect("Unable to lookup data from file.");
                        println!("Table: {:?}", string);
                        let decoded = fm_string_decrypt(&string);
                        res.push(x.to_string() + &decoded);
                    }
                }
            }
            _ => {}
        };

    }
    res
}

fn main() {

    let mut file = HBAMFile::new(Path::new("../fm_vc/databases/Quotes.fmp12"));
    let hbampath = HBAMPath::new(vec!["3", "17"]);
    let leaf = file.get_leaf(&hbampath);

    let leaf_buffer = file.get_buffer_from_leaf(leaf.index.into());
    println!("leaf buffer: {:?}", &leaf_buffer[0..1000]);
    let res = load_tables(&leaf, &leaf_buffer);

    for table in res {
        println!("{}", table);
    }

    println!("index: {}", leaf.index);

}
