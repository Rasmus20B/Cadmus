
pub enum DataSourceType {
    FileMaker,
    ODBC,
}

pub struct DataSource {
    pub id: usize,
    pub name: String,
    pub dstype: DataSourceType,
    pub filename: String,
}

