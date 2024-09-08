use std::path::Path;
use fm_core::file_repr::FmpFileView;

mod fm_core;
mod fm_io;
mod fm_format;
mod util;

fn main() {

    let file = FmpFileView::new(Path::new("test_data/input/blank.fmp12"));
    for table in &file.tables {
        let t = file.get_table(*table.0);
        println!("table: {:?}", t);
    }

}
