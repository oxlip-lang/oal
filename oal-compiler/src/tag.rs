#[derive(PartialEq, Clone, Debug)]
pub struct FuncTag {
    pub bindings: Vec<Tag>,
    pub range: Box<Tag>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Tag {
    Primitive,
    Relation,
    Property,
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
}

pub trait Tagged {
    fn tag(&self) -> Option<&Tag>;
    fn set_tag(&mut self, t: Tag);
    fn unwrap_tag(&self) -> Tag;
    fn with_tag(self, t: Tag) -> Self;
}
