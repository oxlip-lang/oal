use crate::Pair;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{}..{},{}",
            self.start.0, self.start.1, self.end.0, self.end.1
        )
    }
}

impl From<&Pair<'_>> for Span {
    fn from(p: &Pair) -> Self {
        let s = p.as_span();
        Span {
            start: s.start_pos().line_col(),
            end: s.end_pos().line_col(),
        }
    }
}
