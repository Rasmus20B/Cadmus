
use std::collections::HashMap;
use std::cell::RefCell;

#[derive(Clone, Debug)]
pub struct Record {
    id: usize,
    fields: RefCell<HashMap<String, String>>
}

impl Record {
    pub fn new(id_: usize, fields_: &Vec<String>) -> Self {
        Self {
            id: id_,
            fields: RefCell::new(fields_
                .into_iter()
                .zip(vec![String::new(); fields_.len()])
                .map(|(name, val)| (name.clone(), val))
                .collect::<HashMap<_, _>>()),
        }
    }

    pub fn set_field(&self, fieldname: &str, value: &str) {
        let mut handle = self.fields.borrow_mut();
        let field = match handle.get_mut(fieldname) {
            Some(inner) => inner,
            None => return
        };
        *field = value.to_string();
    }

    pub fn get_field(&self, fieldname: &str) -> Option<String> {
        let handle = self.fields.borrow();
        handle.get(fieldname).cloned()
    }
}
