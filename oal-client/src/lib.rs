pub mod cli;
pub mod config;
pub mod lsp;
pub mod utf16;

#[cfg(test)]
mod utf16_tests;

use oal_model::locator::Locator;
use std::fmt::{Display, Formatter};
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

/// A span of Unicode code points.
pub struct CharSpan {
    start: usize,
    end: usize,
    loc: Locator,
}

fn utf8_to_char_index(input: &str, index: usize) -> usize {
    let mut char_index = 0;
    for (utf8_index, _) in input.char_indices() {
        if utf8_index >= index {
            return char_index;
        }
        char_index += 1;
    }
    char_index
}

#[test]
fn test_utf8_to_char_index() {
    let input = "some😉text!";
    assert_eq!(input.len(), 13);
    assert_eq!(input.chars().count(), 10);
    assert_eq!(utf8_to_char_index(input, 0), 0);
    assert_eq!(utf8_to_char_index(input, 8), 5);
    assert_eq!(utf8_to_char_index(input, 42), 10);
}

impl CharSpan {
    fn from(input: &str, span: oal_model::span::Span) -> Self {
        CharSpan {
            start: utf8_to_char_index(input, span.start()),
            end: utf8_to_char_index(input, span.end()),
            loc: span.locator().clone(),
        }
    }
}

impl ariadne::Span for CharSpan {
    type SourceId = Locator;

    fn source(&self) -> &Self::SourceId {
        &self.loc
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

impl Display for CharSpan {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}..{}", self.loc, self.start, self.end)
    }
}
