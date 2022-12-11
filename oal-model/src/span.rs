use std::fmt::{Display, Formatter};

/// Deprecated span type.
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
