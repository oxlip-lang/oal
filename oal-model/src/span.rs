use std::fmt::{Display, Formatter};
use std::ops::Range;

/// The parsing span type.
// TODO: track parsing context
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span(usize, usize);

impl Span {
    pub fn new(range: Range<usize>) -> Self {
        Span(range.start, range.end)
    }

    pub fn range(&self) -> Range<usize> {
        self.0..self.1
    }

    pub fn start(&self) -> usize {
        self.0
    }

    pub fn end(&self) -> usize {
        self.1
    }
}

impl chumsky::Span for Span {
    type Context = ();

    type Offset = usize;

    fn new(_: Self::Context, range: Range<Self::Offset>) -> Self {
        Span::new(range)
    }

    fn context(&self) -> Self::Context {}

    fn start(&self) -> Self::Offset {
        self.start()
    }

    fn end(&self) -> Self::Offset {
        self.end()
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start(), self.end())
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Span::new(range)
    }
}
