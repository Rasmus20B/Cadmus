

// No indirection, we store duplicate instruction names.



pub struct ArgLookupTable([(&'static str, u8, &'static str); 12]);

impl ArgLookupTable {
    pub const fn new() -> Self {
        Self([
            ("set_variable", 0, "var"),
            ("set_variable", 1, "expr"),
            ("set_variable", 2, "rep"),

            ("set_field", 0, "field"),
            ("set_field", 1, "expr"),
            ("set_field", 2, "repetition"),

            ("go_to_layout", 0, "layout"),
            ("go_to_layout", 1, "animation"),

            ("if", 0, "expr"),

            ("elif", 0, "expr"),

            ("exit_loop_if", 0, "expr"),

            ("print", 0, "expr"),
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
