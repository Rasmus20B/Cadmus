use std::{collections::HashMap, path::Path};

use cadmus_objects::schema::Schema;


pub struct SchemaCache {
    map: HashMap<String, Schema>
}

pub struct ProtoSchemaCache {
    map: HashMap<String, Schema>
}
