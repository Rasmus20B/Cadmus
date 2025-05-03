use super::cache;
use super::error::Result;
use crate::cache::ProtoSchemaCache;
use crate::error::Error;
use crate::parser_worker::ParserWorker;
use crate::proto_schema::ProtoSchema;
use cadmus_objects::schema::Schema;
use std::path::Path;

use rayon::prelude::*;

pub fn compile_project(path: &Path) -> Result<Schema> {
    let proto_cache = ProtoSchemaCache::new();
    let paths = std::fs::read_dir(path)
        .map_err(Error::Fs)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect::<Vec<_>>();

    let worker_results = paths
        .par_iter()
        .map(|path| ParserWorker::new().with_path(path.clone()).build())
        .collect::<Vec<_>>();

    Ok(Schema::new())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::super::error::Result;
    use super::compile_project;

    #[test]
    fn basic() {
        let _ = compile_project(Path::new("./test_data/multi_file_solution"));
    }
}
