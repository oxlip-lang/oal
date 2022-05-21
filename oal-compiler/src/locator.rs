use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::io;
use std::path::Path;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Locator {
    path: Rc<Path>,
}

impl Locator {
    pub fn read_to_string(&self) -> io::Result<String> {
        std::fs::read_to_string(&self.path)
    }
}

impl<S: AsRef<OsStr> + ?Sized> From<&S> for Locator {
    fn from(s: &S) -> Self {
        let path = Rc::from(Path::new(s));
        Locator { path }
    }
}

impl Display for Locator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}
