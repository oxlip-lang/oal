use oal_syntax::ast;

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

    pub fn is_status_like(&self) -> bool {
        matches!(self, Tag::Status | Tag::Number)
    }
}

impl From<&ast::Literal> for Tag {
    fn from(l: &ast::Literal) -> Self {
        match l {
            ast::Literal::Text(_) => Tag::Text,
            ast::Literal::Number(_) => Tag::Number,
            ast::Literal::Status(_) => Tag::Status,
        }
    }
}

pub trait Tagged {
    fn tag(&self) -> Option<&Tag>;
    fn set_tag(&mut self, t: Tag);
    fn unwrap_tag(&self) -> Tag;
    fn with_tag(self, t: Tag) -> Self;
}
