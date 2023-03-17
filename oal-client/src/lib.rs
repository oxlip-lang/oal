pub mod config;

use oal_model::locator::Locator;
use std::path::PathBuf;

/// Reads the file at the given location.
pub fn read_file(loc: &Locator) -> anyhow::Result<String> {
    let path: PathBuf = loc.try_into()?;
    let input = std::fs::read_to_string(path)?;
    Ok(input)
}

/// Writes to the file at the given location.
pub fn write_file(loc: &Locator, buf: String) -> anyhow::Result<()> {
    let path: PathBuf = loc.try_into()?;
    std::fs::write(path, buf)?;
    Ok(())
}
