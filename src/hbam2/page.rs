use crate::util::encoding_util::get_int;


#[derive(Clone, Copy, Debug)]
pub enum BlockType {
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
    pub page_type: BlockType,
}

#[derive(Clone, Copy, Debug)]
pub enum PageHeaderErr {
    InvalidPageType(u32, u32)
}

impl PageHeader {
    pub const SIZE: usize = 20;
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PageHeaderErr> {
        let deleted_ = bytes[0] != 0;
        let level_ = bytes[1] as u32 & 0x00FFFFFF;
        let previous_ = get_int(&bytes[4..8]) as u32;
        let next_ = get_int(&bytes[8..12]) as u32;
        let page_type_ = match bytes[13] {
            0 => BlockType::Data,
            1 => BlockType::Index,
            3 => BlockType::Root,
            _ => return Err(PageHeaderErr::InvalidPageType(previous_, next_))
        };
        Ok(Self {
            index: 0,
            deleted: deleted_,
            level: level_,
            previous: previous_,
            next: next_,
            page_type: page_type_,
        })
    }
}

#[derive(Clone, Copy)]
pub struct Page {
    pub header: PageHeader,
    pub data: [u8; 4096],
}

impl Page {
    pub const SIZE: u64 = 4096;

    pub fn from_bytes(bytes: &[u8; 4096]) -> Self {
        Self {
            header: PageHeader::from_bytes(bytes).expect("Unable to read page header."),
            data: *bytes,
        }
    }
}
