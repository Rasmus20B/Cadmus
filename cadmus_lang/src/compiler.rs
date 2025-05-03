use std::path::Path;

use cadmus_objects::schema::Schema;

use super::cache;
use super::error::Result;
use crate::cache::ProtoSchemaCache;
use crate::error::Error;
use crate::proto_schema::ProtoSchema;

fn compile_file_to_proto_schema(path: &Path) -> Result<ProtoSchema> {
    todo!()
}

pub fn compile_project(path: &Path) -> Result<Schema> {
    let proto_cache = ProtoSchemaCache::new();
    for file in std::fs::read_dir(path).map_err(Error::Fs)? {
        match file {
            Ok(inner) => {
                println!("Found file: {:?}", inner);
                // compile_file_to_proto_schema(inner.path().as_path())?;
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }
    Ok(Schema::new())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::super::error::Result;
    use super::compile_project;
}
