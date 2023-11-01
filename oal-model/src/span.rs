use crate::locator::Locator;
use std::fmt::{Display, Formatter};
use std::ops::Range;

/// The parsing span type, expressed as UTF-8 indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    start: usize,
    end: usize,
    loc: Locator,
}

impl Span {
    pub fn new(loc: Locator, range: Range<usize>) -> Self {
        Span {
            start: range.start,
            end: range.end,
            loc,
        }
    }

    pub fn locator(&self) -> &Locator {
        &self.loc
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}..{}", self.loc, self.start, self.end)
    }
}

/// A span of Unicode code points.
pub struct CharSpan {
    pub start: usize,
    pub end: usize,
    pub loc: Locator,
}

/// Converts a UTF-8 index to a Unicode code point index.
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
    pub fn from(input: &str, span: Span) -> Self {
        CharSpan {
            start: utf8_to_char_index(input, span.start()),
            end: utf8_to_char_index(input, span.end()),
            loc: span.locator().clone(),
        }
    }
}

impl Display for CharSpan {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}..{}", self.loc, self.start, self.end)
    }
}
