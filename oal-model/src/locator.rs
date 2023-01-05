use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::rc::Rc;
use std::result::Result;
use url::Url;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid URL")]
    InvalidUrl(#[from] url::ParseError),
    #[error("invalid path")]
    InvalidPath(std::ffi::OsString),
    #[error("input/output error")]
    IO(#[from] std::io::Error),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Locator {
    pub url: Rc<Url>,
}

impl Locator {
    pub fn url(&self) -> &Url {
        self.url.as_ref()
    }

    pub fn join(&self, path: &str) -> Result<Self, Error> {
        let url = self.url.join(path).map(Rc::new)?;
        Ok(Locator { url })
    }
}

impl Debug for Locator {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "<{}>", self.url)
    }
}

impl TryFrom<&str> for Locator {
    type Error = url::ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let url = Url::parse(s).map(Rc::new)?;
        Ok(Locator { url })
    }
}

impl TryFrom<&Path> for Locator {
    type Error = Error;

    fn try_from(p: &Path) -> Result<Self, Self::Error> {
        let path = p.canonicalize()?;
        let url = Url::from_file_path(path)
            .map(Rc::new)
            .map_err(|_| Error::InvalidPath(p.as_os_str().to_owned()))?;
        Ok(Locator { url })
    }
}

impl Display for Locator {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}
