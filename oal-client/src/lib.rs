pub mod cli;
pub mod config;
pub mod lsp;

use oal_model::locator::Locator;
use std::io;
use std::path::PathBuf;

pub trait FileSystem {
    fn read_file(&self, loc: &Locator) -> io::Result<String>;
    fn write_file(&self, loc: &Locator, buf: String) -> io::Result<()>;
}

pub struct DefaultFileSystem;

impl FileSystem for DefaultFileSystem {
    fn read_file(&self, loc: &Locator) -> io::Result<String> {
        let path: PathBuf = loc
            .try_into()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        std::fs::read_to_string(path)
    }

    fn write_file(&self, loc: &Locator, buf: String) -> io::Result<()> {
        let path: PathBuf = loc
            .try_into()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        std::fs::write(path, buf)
    }
}
