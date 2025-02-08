

// No indirection, we store duplicate instruction names.



pub struct ArgLookupTable([(&'static str, u8, &'static str); 2]);

impl ArgLookupTable {
    pub const fn new() -> Self {
        Self([
            ("set_variable", 0, "var"),
            ("set_variable", 1, "expr"),
        ])
    }

    pub fn get_argname(&self, instruction: &str, position: u8) -> Option<&'static str> {
        let entry = self.0.iter()
            .find(|entry| entry.0 == instruction && entry.1 == position);

        match entry {
            Some(inner) => Some(inner.2),
            None => None,
        }
    }
}

pub(crate) const ARG_LOOKUP: ArgLookupTable = ArgLookupTable::new();
