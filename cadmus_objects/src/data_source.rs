
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataSourceType {
    FileMaker,
    Cadmus,
    ODBC,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DataSource {
    pub id: u32,
    pub name: String,
    pub dstype: DataSourceType,
    pub paths: Vec<String>,
}

impl DataSource {
    pub fn to_cad(&self) -> String {
        let mut buffer = format!("extern %{} {} : ", self.id, self.name);
        for path in &self.paths[0..self.paths.len() - 1] {
            buffer.push_str(&format!("{}, ", path));
        }
        buffer.push_str(&format!("{}", self.paths.last().unwrap()));
        buffer
    }
}
