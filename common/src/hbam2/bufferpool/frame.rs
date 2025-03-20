use super::page::{Page, PageGuard};


pub struct Frame  {
    evict: bool,
    references: u32,
    pub page: PageGuard,
}

pub fn get_page(&self) -> PageGuard {
}
