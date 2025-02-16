
pub struct Window {
    pub id: u32,
    pub name: String,
    pub database: String,
    pub layout_id: u32,
}

impl Window {
    pub fn new() -> Self {
        Self {
            id: 0,
            name: String::new(),
            database: String::new(),
            layout_id: 0,
        }
    }

    pub fn id(mut self, id: u32) -> Self {
        self.id = id;
        self
    }

    pub fn name(mut self, name_: String) -> Self {
        self.name = name_;
        self
    }

    pub fn database(mut self, database_: String) -> Self {
        self.database = database_;
        self
    }

    pub fn layout_id(mut self, layout_id: u32) -> Self {
        self.layout_id = layout_id;
        self
    }
}
