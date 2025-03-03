
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
