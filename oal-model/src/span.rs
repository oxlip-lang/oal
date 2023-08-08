use crate::locator::Locator;
use std::fmt::{Display, Formatter};
use std::ops::Range;

/// The parsing span type, expressed as Unicode character indices.
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

impl ariadne::Span for Span {
    type SourceId = Locator;

    fn source(&self) -> &Self::SourceId {
        self.locator()
    }

    fn start(&self) -> usize {
        self.start()
    }

    fn end(&self) -> usize {
        self.end()
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}..{}", self.locator(), self.start(), self.end())
    }
}
