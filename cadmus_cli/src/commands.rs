
use super::error::Result;

use std::path::{Path, PathBuf};

pub fn init_cadmus_repo(path: &PathBuf) -> Result<()> {
    let cad_dir = path.join("cadmus/");
    let test_dir = cad_dir.clone().join("tests");
    let gen_dir = cad_dir.clone().join("gen");
    std::fs::create_dir(cad_dir.clone())?;
    std::fs::create_dir(gen_dir)?;
    std::fs::create_dir(test_dir)?;

    for f in std::fs::read_dir(path)? {
        let f = f?;
        if let Ok(ft) = f.file_type() {
            if ft.is_file() && f.path().ends_with(".fmp12") {
            }
        }
    }
    todo!()
}
