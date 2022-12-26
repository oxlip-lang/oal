use crate::locator::Locator;

#[derive(PartialEq, Clone, Debug)]
pub struct FuncTag {
    pub bindings: Vec<Tag>,
    pub range: Box<Tag>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Tag {
    Text,
    Number,
    Status,
    Primitive,
    Relation,
    PropertyAny, // TODO: obsolete
    Property(Box<Tag>),
    Object,
    Content,
    Transfer,
    Array,
    Uri,
    Any,
    Func(FuncTag),
    Var(usize),
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

pub trait Tagged {
    fn tag(&self) -> Option<&Tag>;
    fn set_tag(&mut self, t: Tag);
    fn unwrap_tag(&self) -> Tag;
    fn with_tag(self, t: Tag) -> Self;
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Seq(Option<Locator>, usize);

impl Seq {
    /// Create a new sequence of tag variables for the given module.
    pub fn new(m: Locator) -> Self {
        Seq(Some(m), 0)
    }

    /// Allocate a new tag variable sequence number.
    pub fn next(&mut self) -> usize {
        let n = self.1;
        self.1 += 1;
        n
    }

    /// Returns the number of tag variables allocated.
    pub fn len(&self) -> usize {
        self.1
    }
}
