
mod error;

use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::config;

pub type Db = Pool<Postgres>;

