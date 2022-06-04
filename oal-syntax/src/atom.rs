use crate::errors::{Error, Result};
use enum_map::Enum;
use std::num::NonZeroU16;
use std::rc::Rc;

pub type Text = Rc<str>;
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

impl TryFrom<u64> for HttpStatus {
    type Error = Error;

    fn try_from(v: u64) -> Result<Self> {
        if (100..=599).contains(&v) {
            Ok(HttpStatus::Code(unsafe {
                NonZeroU16::new_unchecked(v.try_into().unwrap())
            }))
        } else {
            Err(Error::new("status not in range"))
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Primitive {
    Number,
    String,
    Boolean,
    Integer,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Enum)]
pub enum Method {
    Get,
    Put,
    Post,
    Patch,
    Delete,
    Options,
    Head,
}
