
use common::{cadlang, hbam2};

use super::error::Result;

use std::{io::{BufWriter, Write}, path::{Path, PathBuf}};

pub fn init_cadmus_repo(path: &PathBuf) -> Result<()> {
    let cad_dir = path.join("cadmus/");
    let test_dir = cad_dir.clone().join("tests");
    let gen_dir = cad_dir.clone().join("gen");
    std::fs::create_dir(cad_dir.clone())?;
    std::fs::create_dir(gen_dir.clone())?;
    std::fs::create_dir(test_dir.clone())?;

    println!("Path: {:?}", path);
    let mut files = vec![];
    for f in std::fs::read_dir(path)? {
        let f = f?;
        println!("is {:?} an fmp12?", f.path());
        if f.path().extension().is_some_and(|e| e == "fmp12") {
            println!("it is!");
            let mut hbam_ctx = hbam2::Context::new();
            files.push((f.path(), hbam_ctx.get_schema_contents(f.path().to_str().unwrap())));
        }
    }

    for (path, file) in &files {
        let mut path = path.clone();
        path.set_extension("cad");
        let path = gen_dir.to_path_buf().join(Path::new(&path.file_name().unwrap()));
        println!("writing {:?}...", path);
        let handle = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(path).unwrap();

        let mut writer = BufWriter::new(handle);
        writer.write_all(file.to_cad_with_externs(&files.iter().map(|f| f.1.clone()).collect::<Vec<_>>()).as_bytes())?;
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::init_cadmus_repo;

    #[test]
    fn basic_init_test() {
        let res = init_cadmus_repo(&PathBuf::from("./fmp_project/"));
        println!("{:?}", res);
    }
}
