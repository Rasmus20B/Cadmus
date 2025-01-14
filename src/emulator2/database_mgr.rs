
use super::database::Database;
use std::collections::HashMap;

pub struct DatabaseMgr {
    databases: HashMap<String, Database>
}
