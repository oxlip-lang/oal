pub mod cli;
pub mod config;
pub mod lsp;

use oal_model::locator::Locator;
use std::io;
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid path: {0}")]
    InvalidPath(String),
    #[error("input/output error")]
    IO(#[from] std::io::Error),
}

pub trait FileSystem {
    fn is_valid(&self, loc: &Locator) -> bool;
    fn open_file(&self, loc: &Locator) -> Result<Box<dyn io::Read>, Error>;
    fn read_file(&self, loc: &Locator) -> Result<String, Error>;
    fn write_file(&self, loc: &Locator, buf: String) -> Result<(), Error>;
}

pub struct DefaultFileSystem;

fn locator_path(loc: &Locator) -> Result<PathBuf, Error> {
    let url = loc.url();
    if url.scheme() == "file" {
        if let Ok(p) = url.to_file_path() {
            return Ok(p);
        }
    }
    Err(Error::InvalidPath(url.as_str().to_owned()))
}

impl FileSystem for DefaultFileSystem {
    fn is_valid(&self, loc: &Locator) -> bool {
        match locator_path(loc) {
            Ok(p) => p.exists(),
            Err(_) => false,
        }
    }

    fn open_file(&self, loc: &Locator) -> Result<Box<dyn io::Read>, Error> {
        let path = locator_path(loc)?;
        let file = std::fs::File::open(path)?;
        Ok(Box::new(file))
    }

    fn read_file(&self, loc: &Locator) -> Result<String, Error> {
        let path = locator_path(loc)?;
        let string = std::fs::read_to_string(path)?;
        Ok(string)
    }

    fn write_file(&self, loc: &Locator, buf: String) -> Result<(), Error> {
        let path = locator_path(loc)?;
        std::fs::write(path, buf)?;
        Ok(())
    }
}
