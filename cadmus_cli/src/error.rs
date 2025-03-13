
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FsErr(std::io::Error)
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::FsErr(value)
    }
}

impl std::error::Error for Error {}
