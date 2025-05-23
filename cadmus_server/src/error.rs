
use crate::model;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConfigMissingEnv(&'static str),
    ConfigWrongFormat(&'static str),
    Model(model::Error),
}

impl From<model::Error> for Error {
    fn from(value: model::Error) -> Self {
        Self::Model(value)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
