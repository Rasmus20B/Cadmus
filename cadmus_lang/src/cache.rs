use std::{collections::HashMap, path::Path};

use cadmus_objects::schema::Schema;

use crate::proto_schema::ProtoSchema;

pub type ProtoSchemaCache = HashMap<String, ProtoSchema>;
pub type SchemaCache = HashMap<String, Schema>;
