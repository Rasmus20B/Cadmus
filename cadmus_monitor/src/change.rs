use std::future::Future;

#[derive(PartialEq, Eq)]
pub struct Change {
}
unsafe impl Send for Change {}
unsafe impl Sync for Change {}
