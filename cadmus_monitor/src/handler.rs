
use super::change::Change;
use std::{pin::Pin, future::Future};

use common::hbam2;

type HandlerOutput<'a> = Pin<Box<dyn Future<Output = Result<Change, HandlerError>> + Send + 'a>>;

#[derive(Debug)]
pub enum HandlerError {
    Io(std::io::Error),
    Decode(String),
}

pub trait Handler: Send + Sync + Clone {
    fn handle_modification<'a>(&'a self, path: &str) -> Result<Change, HandlerError>;
}

#[derive(Default, Clone)]
pub struct DummyHandler;

impl Handler for DummyHandler {
    fn handle_modification(&self, path: &str) -> Result<Change, HandlerError> {
        println!("INSIDE HANDLER");
        Ok(Change{})
    }
}

#[derive(Default, Clone)]
pub struct FMPHandler;

impl Handler for FMPHandler {
    fn handle_modification<'a>(&'a self, path: &str) -> Result<Change, HandlerError> {
        let mut ctx = hbam2::Context::new();
        let current_file = ctx.get_schema_contents(path);
        Ok(Change{})
    }
}
