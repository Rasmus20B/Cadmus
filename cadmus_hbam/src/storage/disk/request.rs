
use crate::storage::path::Path;

pub enum ChunkStorage {
    Simple(Vec<u8>),
    SimpleKeyVal(u8, Vec<u8>),
    LongKeyVal(Vec<u8>, Vec<u8>),
}

pub enum Request {
    Write { path: Path, data: ChunkStorage },
    Read { start: Path, end: Path },
}
