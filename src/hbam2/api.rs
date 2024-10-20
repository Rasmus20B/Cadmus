
pub type Key = Vec<String>;
pub type Value = String;

pub struct KeyValue {
    key: String,
    value: String,
}


pub enum APIErr {
    KeyNotFound(Key)
}

pub fn get_keyvalue(key: Key) -> Result<KeyValue, APIErr> {
    unimplemented!()
}

pub fn put_keyvalue(key: Key, val: Value) -> Result<(), APIErr> {
    unimplemented!()
}
