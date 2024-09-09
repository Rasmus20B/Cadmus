
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockLocation {
    pub chunk_index: u32,
    pub offset: u32,
}

impl BlockLocation {
    pub fn new(chunk_index_: u32, local_index_: u32) -> Self {
        Self {
            chunk_index: chunk_index_,
            offset: local_index_,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fm_io::block_location::BlockLocation;

    pub fn comparison_test() {
        assert!(
            BlockLocation { chunk_index: 0, offset: 500 }
            >
            BlockLocation { chunk_index: 0, offset: 200 }
        );
        assert!(
            BlockLocation { chunk_index: 3, offset: 100 }
            >
            BlockLocation { chunk_index: 0, offset: 200 }
        );
        assert!(
            BlockLocation { chunk_index: 1, offset: 500 }
            <
            BlockLocation { chunk_index: 3, offset: 200 }
        );
    }
}

