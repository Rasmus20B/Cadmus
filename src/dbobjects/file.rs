
use super::schema::Schema;
use super::scripting::script::Script;
use super::layout::Layout;
use super::data_source::*;

#[derive(Debug, PartialEq)]
pub struct File {
    pub name: String,
    pub schema: Schema,
    pub layouts: Vec<Layout>,
    pub data_sources: Vec<DataSource>,
    pub scripts: Vec<Script>,
    pub tests: Vec<Script>,
}
