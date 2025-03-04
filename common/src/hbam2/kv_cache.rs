use super::Key;

pub struct CachedKeyLocation {
    pub key: Key,
    pub block_id: u16,
    pub offset: u16,
    padding: u16,
}

pub struct KeyValCache {
    cache: [CachedKeyLocation; 512],
}

