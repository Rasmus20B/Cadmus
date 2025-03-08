
mod base;
mod error;
pub mod project;
pub mod user;
mod store;
use store::{new_db_pool, Db};

pub use self::error::{Error, Result};

#[derive(Clone)]
pub struct ModelManager {
    db: Db,
}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        let db = new_db_pool().await?;
        Ok(ModelManager {
            db
        })
    }

    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }
}
