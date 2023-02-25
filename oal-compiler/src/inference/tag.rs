use oal_model::locator::Locator;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TagId {
    loc: Locator,
    n: usize,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct FuncTag {
    pub bindings: Vec<Tag>,
    pub range: Box<Tag>,
}

impl Display for FuncTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for b in self.bindings.iter() {
            write!(f, "{b} -> ")?;
        }
        write!(f, "{}", self.range)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Tag {
    Text,
    Number,
    Status,
    Primitive,
    Relation,
    Object,
    Content,
    Transfer,
    Array,
    Uri,
    Any,
    Property(Box<Tag>),
    Func(FuncTag),
    Var(TagId),
}

impl Tag {
    pub fn is_variable(&self) -> bool {
        matches!(self, Tag::Var(_))
    }

    pub fn is_schema(&self) -> bool {
        matches!(
            self,
            Tag::Primitive | Tag::Relation | Tag::Object | Tag::Array | Tag::Uri | Tag::Any
        )
    }

    pub fn is_schema_like(&self) -> bool {
        *self == Tag::Content || self.is_schema()
    }

    pub fn is_status_like(&self) -> bool {
        matches!(self, Tag::Status | Tag::Number)
    }

    pub fn is_relation_like(&self) -> bool {
        matches!(self, Tag::Relation | Tag::Uri)
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tag::Text => write!(f, "text"),
            Tag::Number => write!(f, "number"),
            Tag::Status => write!(f, "http status"),
            Tag::Primitive => write!(f, "primitive"),
            Tag::Relation => write!(f, "relation"),
            Tag::Object => write!(f, "object"),
            Tag::Content => write!(f, "content"),
            Tag::Transfer => write!(f, "transfer"),
            Tag::Array => write!(f, "array"),
            Tag::Uri => write!(f, "uri"),
            Tag::Any => write!(f, "any"),
            Tag::Property(t) => write!(f, "property[{t}]"),
            Tag::Func(t) => write!(f, "function[{t}]"),
            Tag::Var(_) => write!(f, "<unknown>"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Seq(TagId);

impl Seq {
    /// Create a new sequence of tag variables for the given module.
    pub fn new(loc: Locator) -> Self {
        Seq(TagId { loc, n: 0 })
    }

    /// Allocate a new tag variable sequence number.
    pub fn next(&mut self) -> TagId {
        let n = self.0.n;
        self.0.n += 1;
        TagId {
            loc: self.0.loc.clone(),
            n,
        }
    }

    /// Returns the number of tag variables allocated.
    pub fn len(&self) -> usize {
        self.0.n
    }
}
