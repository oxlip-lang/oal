use std::num::NonZeroU16;
use std::rc::Rc;

pub type Literal = Rc<str>;
pub type Ident = Rc<str>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum HttpStatusRange {
    Info,
    Success,
    Redirect,
    ClientError,
    ServerError,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum HttpStatus {
    Code(NonZeroU16),
    Range(HttpStatusRange),
}
