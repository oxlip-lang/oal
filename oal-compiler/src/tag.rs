#[derive(PartialEq, Clone, Debug)]
pub struct FuncTag {
    pub bindings: Vec<Tag>,
    pub range: Box<Tag>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Tag {
    Primitive,
    Relation,
    Object,
    Content,
    Array,
    Uri,
    Any,
    Func(FuncTag),
    Var(usize),
}

impl Tag {
    pub fn is_variable(&self) -> bool {
        match self {
            Tag::Var(_) => true,
            _ => false,
        }
    }

    pub fn is_schema(&self) -> bool {
        match self {
            Tag::Primitive | Tag::Relation | Tag::Object | Tag::Array | Tag::Uri | Tag::Any => true,
            _ => false,
        }
    }
}

pub trait Tagged {
    fn tag(&self) -> Option<&Tag>;
    fn set_tag(&mut self, t: Tag);
    fn unwrap_tag(&self) -> Tag;
    fn with_tag(self, t: Tag) -> Self;
}
