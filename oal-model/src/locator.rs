use std::fmt::{Debug, Display, Formatter};
use std::result::Result;
use std::sync::Arc;
use url::Url;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid URL")]
    InvalidUrl(#[from] url::ParseError),
    #[error("empty path")]
    EmptyPath,
}

/// A file locator backed by a URL.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Locator {
    url: Arc<Url>,
}

impl Locator {
    /// Returns a reference to the underlying URL.
    pub fn url(&self) -> &Url {
        self.url.as_ref()
    }

    /// Make a base locator by appending a trailing URL segment separator.
    pub fn as_base(&self) -> Self {
        let mut url = self.url.as_ref().clone();
        // The original URL can be a base by construction so path_segments_mut should never fail.
        url.path_segments_mut().unwrap().push("");
        Locator { url: Arc::new(url) }
    }

    /// Appends a relative path to the locator base.
    pub fn join(&self, path: &str) -> Result<Self, Error> {
        if path.is_empty() {
            Err(Error::EmptyPath)
        } else {
            let url = self.url.join(path).map(Arc::new)?;
            Ok(Locator { url })
        }
    }
}

impl Debug for Locator {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "<{}>", self.url)
    }
}

impl From<Url> for Locator {
    fn from(url: Url) -> Self {
        Locator { url: Arc::new(url) }
    }
}

impl TryFrom<&str> for Locator {
    type Error = url::ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let url = Url::parse(s).map(Arc::new)?;
        Ok(Locator { url })
    }
}

impl Display for Locator {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}
