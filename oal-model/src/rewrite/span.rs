use chumsky::prelude::*;
use std::{
    fmt::{Display, Formatter},
    ops::Range,
};

#[derive(Clone, Debug)]
pub struct Span(Range<usize>);

impl chumsky::Span for Span {
    type Context = ();

    type Offset = usize;

    fn new(_: Self::Context, range: Range<Self::Offset>) -> Self {
        Span(range)
    }

    fn context(&self) -> Self::Context {}

    fn start(&self) -> Self::Offset {
        self.0.start
    }

    fn end(&self) -> Self::Offset {
        self.0.end
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start(), self.end())
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Span::new((), range)
    }
}
