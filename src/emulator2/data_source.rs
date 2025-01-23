
pub enum DataSourceType {
    FileMaker,
    Cadmus,
    ODBC,
}

pub struct DataSource {
    pub id: usize,
    pub name: String,
    pub dstype: DataSourceType,
    pub filename: String,
}

