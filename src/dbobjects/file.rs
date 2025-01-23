
use super::schema::Schema;
use super::scripting::script::Script;

pub struct File {
    name: String,
    schema: Schema,
    scripts: Vec<Script>
}
