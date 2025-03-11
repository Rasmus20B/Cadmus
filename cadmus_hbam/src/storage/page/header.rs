
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageType {
    Root,
    Index,
    Data,
}

#[derive(Clone, Copy, Debug)]
pub struct PageHeader {
    pub index: u32,
    pub deleted: bool,
    pub level: u32,
    pub previous: u32,
    pub next: u32,
    pub page_type: PageType,
}

impl PageHeader {
    pub const SIZE: u64 = 20;
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let deleted = bytes[0] != 0;
        let level = bytes[1] as u32 & 0x00FFFFFF;
        let previous = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
        let next = u32::from_be_bytes(bytes[8..12].try_into().unwrap());
        let page_type = match bytes[13] {
            0 => PageType::Data,
            _ => PageType::Index,
        };
        Self {
            index: 0,
            deleted,
            level,
            previous,
            next,
            page_type,
        }
    }
}

