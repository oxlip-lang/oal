use crate::errors::{Error, Result};
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::rc::Rc;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Locator {
    pub url: Rc<Url>,
}

impl Locator {
    pub fn join(&self, path: &str) -> Result<Locator> {
        let url = self.url.join(path).map(Rc::new)?;
        Ok(Locator { url })
    }
}

impl TryFrom<&str> for Locator {
    type Error = Error;

    fn try_from(s: &str) -> Result<Locator> {
        let url = Url::parse(s).map(Rc::new)?;
        Ok(Locator { url })
    }
}

impl TryFrom<&Path> for Locator {
    type Error = Error;

    fn try_from(p: &Path) -> Result<Locator> {
        let path = p.canonicalize()?;
        let url = Url::from_file_path(path).map(Rc::new)?;
        Ok(Locator { url })
    }
}

impl Display for Locator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}
