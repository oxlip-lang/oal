use crate::atom;
use crate::lexer::{Token, TokenKind, TokenValue};
use oal_model::grammar::*;
use oal_model::lexicon::Cursor;
use oal_model::{syntax_nodes, terminal_node};
use std::fmt::Debug;

#[cfg(test)]
use oal_model::lexicon::{Lexeme, TokenList};
#[cfg(test)]
use oal_model::locator::Locator;

#[derive(Copy, Clone, Default, Debug)]
pub struct Gram;

impl Grammar for Gram {
    type Lex = Token;
    type Kind = SyntaxKind;
    type Tag = ParserTag;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ParserTag {
    Term,
    Expression,
}

terminal_node!(Gram, Identifier, k if k.is_identifier());

impl<T: Core> Identifier<'_, T> {
    pub fn ident(&self) -> atom::Ident {
        self.node().as_str().into()
    }
}

impl<T: Core> PartialEq for Identifier<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.node().as_str() == other.node().as_str()
    }
}

terminal_node!(Gram, Primitive, k if k.is_primitive());

#[derive(Debug, PartialEq, Eq)]
pub enum PrimitiveKind {
    Num,
    Str,
    Uri,
    Bool,
    Int,
}

impl<T: Core> Primitive<'_, T> {
    pub fn kind(&self) -> PrimitiveKind {
        match self.node().token().kind() {
            TokenKind::PrimitiveNum => PrimitiveKind::Num,
            TokenKind::PrimitiveStr => PrimitiveKind::Str,
            TokenKind::PrimitiveUri => PrimitiveKind::Uri,
            TokenKind::PrimitiveBool => PrimitiveKind::Bool,
            TokenKind::PrimitiveInt => PrimitiveKind::Int,
            k => unreachable!("not a literal {:?}", k),
        }
    }
}

terminal_node!(Gram, PathElement, k if k.is_path_element());

impl<'a, T: Core> PathElement<'a, T> {
    pub fn as_str(&self) -> &'a str {
        match self.node().token().kind() {
            // Note that path separators are omitted from the string representation.
            TokenKind::PathElementRoot => "",
            TokenKind::PathElementSegment => self.node().as_str(),
            _ => unreachable!(),
        }
    }
}

terminal_node!(Gram, PropertyName, TokenKind::Property);

impl<T: Core> PropertyName<'_, T> {
    pub fn as_text(&self) -> atom::Text {
        self.node().as_str().into()
    }
}

terminal_node!(Gram, Method, k if k.is_method());

impl<T: Core> Method<'_, T> {
    pub fn method(&self) -> atom::Method {
        match self.node().token().kind() {
            TokenKind::MethodGet => atom::Method::Get,
            TokenKind::MethodPut => atom::Method::Put,
            TokenKind::MethodPost => atom::Method::Post,
            TokenKind::MethodPatch => atom::Method::Patch,
            TokenKind::MethodDelete => atom::Method::Delete,
            TokenKind::MethodOptions => atom::Method::Options,
            TokenKind::MethodHead => atom::Method::Head,
            _ => unreachable!(),
        }
    }
}

terminal_node!(Gram, Literal, k if k.is_literal());

#[derive(Debug, PartialEq, Eq)]
pub enum LiteralKind {
    HttpStatus,
    Number,
    String,
}

impl<'a, T: Core> Literal<'a, T> {
    pub fn kind(&self) -> LiteralKind {
        match self.node().token().kind() {
            TokenKind::LiteralHttpStatus => LiteralKind::HttpStatus,
            TokenKind::LiteralNumber => LiteralKind::Number,
            TokenKind::LiteralString => LiteralKind::String,
            k => unreachable!("not a literal {:?}", k),
        }
    }

    pub fn value(&self) -> &'a TokenValue {
        self.node().token().value()
    }

    pub fn as_str(&self) -> &'a str {
        self.node().as_str()
    }
}

terminal_node!(Gram, ContentTag, k if k.is_content());

#[derive(Debug, PartialEq, Eq)]
pub enum ContentTagKind {
    Media,
    Headers,
    Status,
}

impl<T: Core> ContentTag<'_, T> {
    pub fn kind(&self) -> ContentTagKind {
        match self.node().token().kind() {
            TokenKind::ContentHeaders => ContentTagKind::Headers,
            TokenKind::ContentMedia => ContentTagKind::Media,
            TokenKind::ContentStatus => ContentTagKind::Status,
            k => unreachable!("not a content tag {:?}", k),
        }
    }
}

terminal_node!(Gram, Operator, k if k.is_operator());

impl<T: Core> Operator<'_, T> {
    pub fn variadic(&self) -> atom::VariadicOperator {
        match self.node().token().kind() {
            TokenKind::OperatorDoubleColon => atom::VariadicOperator::Range,
            TokenKind::OperatorAmpersand => atom::VariadicOperator::Join,
            TokenKind::OperatorTilde => atom::VariadicOperator::Any,
            TokenKind::OperatorVerticalBar => atom::VariadicOperator::Sum,
            op => unreachable!("not a variadic operator {:?}", op),
        }
    }

    pub fn unary(&self) -> atom::UnaryOperator {
        match self.node().token().kind() {
            TokenKind::OperatorExclamationMark => atom::UnaryOperator::Required,
            TokenKind::OperatorQuestionMark => atom::UnaryOperator::Optional,
            op => unreachable!("not an unary operator {:?}", op),
        }
    }
}

terminal_node!(
    Gram,
    OptionMark,
    TokenKind::OperatorExclamationMark | TokenKind::OperatorQuestionMark
);

impl<T: Core> OptionMark<'_, T> {
    pub fn required(&self) -> bool {
        match self.node().token().kind() {
            TokenKind::OperatorExclamationMark => true,
            TokenKind::OperatorQuestionMark => false,
            op => unreachable!("not an option mark {:?}", op),
        }
    }
}

terminal_node!(
    Gram,
    Annotation,
    TokenKind::AnnotationLine | TokenKind::AnnotationInline
);

impl<'a, T: Core> Annotation<'a, T> {
    pub fn as_str(&self) -> &'a str {
        self.node().as_str()
    }
}

// TODO: add support for document attributes
syntax_nodes!(
    Gram,
    Terminal,
    SubExpression,
    Variable,
    ContentMeta,
    ContentMetaList,
    ContentBody,
    Content,
    Property,
    Array,
    Annotations,
    Bindings,
    Binding,
    Declaration,
    UriVariable,
    UriPath,
    UriParams,
    UriTemplate,
    PropertyList,
    Object,
    Application,
    VariadicOp,
    UnaryOp,
    XferMethods,
    XferParams,
    XferDomain,
    Transfer,
    Import,
    Qualifier,
    Resource,
    XferList,
    Relation,
    Recursion,
    Program,
    Error
);

impl<'a, T: Core> Program<'a, T> {
    pub fn resources(&self) -> impl Iterator<Item = Resource<'a, T>> {
        self.node().children().filter_map(Resource::cast)
    }

    pub fn declarations(&self) -> impl Iterator<Item = Declaration<'a, T>> {
        self.node().children().filter_map(Declaration::cast)
    }

    pub fn imports(&self) -> impl Iterator<Item = Import<'a, T>> {
        self.node().children().filter_map(Import::cast)
    }
}

impl<'a, T: Core> Resource<'a, T> {
    const RELATION_POS: usize = 1;

    pub fn relation(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::RELATION_POS)
    }
}

impl<'a, T: Core> Annotations<'a, T> {
    pub fn items(&self) -> impl Iterator<Item = Annotation<'a, T>> {
        self.node().children().filter_map(Annotation::cast)
    }
}

impl<T: Core> Binding<'_, T> {
    pub fn ident(&self) -> atom::Ident {
        Identifier::cast(self.node().first())
            .expect("expected identifier")
            .ident()
    }
}

impl<'a, T: Core> Declaration<'a, T> {
    const ANNOTATIONS_POS: usize = 0;
    const IDENTIFIER_POS: usize = 2;
    const BINDINGS_POS: usize = 3;
    const RHS_POS: usize = 5;

    pub fn annotations(&self) -> impl Iterator<Item = Annotation<'a, T>> {
        Annotations::cast(self.node().nth(Self::ANNOTATIONS_POS))
            .expect("expected annotations")
            .items()
    }

    pub fn identifier(&self) -> Identifier<'a, T> {
        Identifier::cast(self.node().nth(Self::IDENTIFIER_POS))
            .expect("declaration lhs must be an identifier")
    }

    pub fn ident(&self) -> atom::Ident {
        self.identifier().ident()
    }

    pub fn bindings(&self) -> impl Iterator<Item = Binding<'a, T>> {
        self.node()
            .nth(Self::BINDINGS_POS)
            .children()
            .filter_map(Binding::cast)
    }

    pub fn has_bindings(&self) -> bool {
        self.bindings().next().is_some()
    }

    pub fn rhs(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::RHS_POS)
    }
}

impl<'a, T: Core> Qualifier<'a, T> {
    const IDENTIFIER_POS: usize = 1;

    pub fn ident(&self) -> Option<atom::Ident> {
        self.identifier().map(|i| i.ident())
    }

    pub fn identifier(&self) -> Option<Identifier<'a, T>> {
        self.node()
            .children()
            .nth(Self::IDENTIFIER_POS)
            .map(|n| Identifier::cast(n).expect("qualifier must be an identifier"))
    }
}

impl<'a, T: Core> Import<'a, T> {
    const MODULE_POS: usize = 1;
    const QUALIFIER_POS: usize = 2;

    pub fn module(&self) -> &'a str {
        self.node().nth(Self::MODULE_POS).as_str()
    }

    pub fn qualifier(&self) -> Option<atom::Ident> {
        Qualifier::cast(self.node().nth(Self::QUALIFIER_POS))
            .expect("expected qualifier")
            .ident()
    }
}

impl<'a, T: Core> Terminal<'a, T> {
    const PREFIX_ANN_POS: usize = 0;
    const INNER_POS: usize = 1;
    const SUFFIX_ANN_POS: usize = 2;

    pub fn inner(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::INNER_POS)
    }

    pub fn prefix_annotations(&self) -> impl Iterator<Item = Annotation<'a, T>> {
        Annotations::cast(self.node().nth(Self::PREFIX_ANN_POS))
            .expect("expected annotations")
            .items()
    }

    pub fn suffix_annotation(&self) -> Option<Annotation<'a, T>> {
        self.node()
            .children()
            .nth(Self::SUFFIX_ANN_POS)
            .map(|n| Annotation::cast(n).expect("expected annotation"))
    }

    pub fn annotations(&self) -> impl Iterator<Item = Annotation<'a, T>> {
        self.prefix_annotations().chain(self.suffix_annotation())
    }
}

impl<'a, T: Core> Array<'a, T> {
    const INNER_POS: usize = 1;

    pub fn inner(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::INNER_POS)
    }
}

impl<'a, T: Core> UriVariable<'a, T> {
    const INNER_POS: usize = 2;

    pub fn inner(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::INNER_POS)
    }
}

impl<'a, T: Core> Recursion<'a, T> {
    const BINDING_POS: usize = 1;
    const RHS_POS: usize = 2;

    pub fn binding(&self) -> Binding<'a, T> {
        Binding::cast(self.node().nth(Self::BINDING_POS)).expect("should be a binding")
    }

    pub fn rhs(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::RHS_POS)
    }
}

#[derive(Debug)]
pub enum UriSegment<'a, T: Core> {
    Element(PathElement<'a, T>),
    Variable(UriVariable<'a, T>),
}

impl<'a, T: Core> UriPath<'a, T> {
    #[allow(clippy::manual_map)]
    pub fn segments(&self) -> impl Iterator<Item = UriSegment<'a, T>> {
        self.node().children().filter_map(|c| {
            if let Some(v) = UriVariable::cast(c) {
                Some(UriSegment::Variable(v))
            } else if let Some(p) = PathElement::cast(c) {
                Some(UriSegment::Element(p))
            } else {
                None
            }
        })
    }
}

impl<'a, T: Core> UriTemplate<'a, T> {
    const PATH_POS: usize = 0;
    const PARAMS_POS: usize = 1;

    pub fn segments(&self) -> impl Iterator<Item = UriSegment<'a, T>> {
        UriPath::cast(self.node().nth(Self::PATH_POS))
            .expect("expected an URI path")
            .segments()
    }

    pub fn params(&self) -> Option<Object<'a, T>> {
        self.node().children().nth(Self::PARAMS_POS).map(|inner| {
            UriParams::cast(inner)
                .expect("expected URI parameters")
                .inner()
        })
    }
}

impl<'a, T: Core> UriParams<'a, T> {
    const INNER_POS: usize = 1;

    pub fn inner(&self) -> Object<'a, T> {
        Object::cast(self.node().nth(Self::INNER_POS)).expect("URI parameters must be an object")
    }
}

impl<'a, T: Core> Property<'a, T> {
    const OPTION_POS: usize = 1;

    pub fn name(&self) -> atom::Text {
        PropertyName::cast(self.node().first())
            .expect("expected a property name")
            .as_text()
    }

    pub fn required(&self) -> Option<bool> {
        self.node()
            .children()
            .nth(Self::OPTION_POS)
            .and_then(OptionMark::cast)
            .map(|m| m.required())
    }

    pub fn rhs(&self) -> NodeRef<'a, T, Gram> {
        self.node()
            .children()
            .last()
            .expect("expected a right-hand side")
    }
}

impl<'a, T: Core> PropertyList<'a, T> {
    pub fn items(&self) -> impl Iterator<Item = NodeRef<'a, T, Gram>> {
        self.node().children().step_by(2)
    }
}

impl<'a, T: Core> Object<'a, T> {
    const PROPS_POS: usize = 1;

    pub fn properties(&self) -> impl Iterator<Item = NodeRef<'a, T, Gram>> {
        PropertyList::cast(self.node().nth(Self::PROPS_POS))
            .expect("expected property list")
            .items()
    }
}

impl<'a, T: Core> XferMethods<'a, T> {
    pub fn methods(&self) -> impl Iterator<Item = atom::Method> + 'a {
        self.node()
            .children()
            .filter_map(Method::cast)
            .map(|m| m.method())
    }
}

impl<'a, T: Core> XferParams<'a, T> {
    const INNER_POS: usize = 0;

    #[allow(clippy::iter_nth_zero)]
    pub fn inner(&self) -> Option<Object<'a, T>> {
        self.node()
            .children()
            .nth(Self::INNER_POS)
            .map(|inner| Object::cast(inner).expect("transfer parameters must be an object"))
    }
}

impl<'a, T: Core> XferDomain<'a, T> {
    const INNER_POS: usize = 1;

    pub fn inner(&self) -> Option<Terminal<'a, T>> {
        self.node()
            .children()
            .nth(Self::INNER_POS)
            .map(|inner| Terminal::cast(inner).expect("transfer domain must be a terminal"))
    }
}

impl<'a, T: Core> Transfer<'a, T> {
    const METHODS_POS: usize = 0;
    const PARAMS_POS: usize = 1;
    const DOMAIN_POS: usize = 2;
    const RANGE_POS: usize = 4;

    pub fn methods(&self) -> impl Iterator<Item = atom::Method> + 'a {
        XferMethods::cast(self.node().nth(Self::METHODS_POS))
            .expect("expected transfer methods")
            .methods()
    }

    pub fn params(&self) -> Option<Object<'a, T>> {
        XferParams::cast(self.node().nth(Self::PARAMS_POS))
            .expect("expected transfer parameters")
            .inner()
    }

    pub fn domain(&self) -> Option<Terminal<'a, T>> {
        XferDomain::cast(self.node().nth(Self::DOMAIN_POS))
            .expect("expected transfer domain")
            .inner()
    }

    pub fn range(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::RANGE_POS)
    }
}

impl<'a, T: Core> VariadicOp<'a, T> {
    const OPERATOR_POS: usize = 1;

    pub fn operator(&self) -> atom::VariadicOperator {
        Operator::cast(self.node().nth(Self::OPERATOR_POS))
            .expect("expected an operator")
            .variadic()
    }

    pub fn operands(&self) -> impl Iterator<Item = NodeRef<'a, T, Gram>> {
        self.node().children().step_by(2)
    }
}

impl<'a, T: Core> UnaryOp<'a, T> {
    const OPERATOR_POS: usize = 1;

    pub fn operator(&self) -> atom::UnaryOperator {
        Operator::cast(self.node().nth(Self::OPERATOR_POS))
            .expect("expected an operator")
            .unary()
    }

    pub fn operand(&self) -> NodeRef<'a, T, Gram> {
        self.node().first()
    }
}

impl<'a, T: Core> ContentMeta<'a, T> {
    const TAG_POS: usize = 0;
    const RHS_POS: usize = 2;

    pub fn kind(&self) -> ContentTagKind {
        ContentTag::cast(self.node().nth(Self::TAG_POS))
            .expect("expected content tag")
            .kind()
    }

    pub fn rhs(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::RHS_POS)
    }
}

impl<'a, T: Core> ContentMetaList<'a, T> {
    pub fn items(&self) -> impl Iterator<Item = ContentMeta<'a, T>> {
        self.node().children().filter_map(ContentMeta::cast)
    }
}

impl<'a, T: Core> ContentBody<'a, T> {
    pub fn inner(&self) -> NodeRef<'a, T, Gram> {
        self.node().first()
    }
}

impl<'a, T: Core> Content<'a, T> {
    pub fn meta(&self) -> Option<impl Iterator<Item = ContentMeta<'a, T>>> {
        self.node()
            .children()
            .find_map(ContentMetaList::cast)
            .map(|m| m.items())
    }

    pub fn body(&self) -> Option<NodeRef<'a, T, Gram>> {
        self.node()
            .children()
            .find_map(ContentBody::cast)
            .map(|m| m.inner())
    }
}

impl<'a, T: Core> Application<'a, T> {
    pub fn lambda(&self) -> Variable<'a, T> {
        Variable::cast(self.node().first()).expect("expected a variable")
    }

    pub fn arguments(&self) -> impl Iterator<Item = Terminal<'a, T>> {
        self.node().children().skip(1).filter_map(Terminal::cast)
    }
}

impl<'a, T: Core> XferList<'a, T> {
    pub fn items(&self) -> impl Iterator<Item = NodeRef<'a, T, Gram>> {
        self.node().children().step_by(2)
    }
}

impl<'a, T: Core> Relation<'a, T> {
    const URI_POS: usize = 0;
    const XFERS_POS: usize = 2;

    pub fn uri(&self) -> Terminal<'a, T> {
        Terminal::cast(self.node().nth(Self::URI_POS)).expect("expected a terminal")
    }

    pub fn transfers(&self) -> impl Iterator<Item = NodeRef<'a, T, Gram>> {
        XferList::cast(self.node().nth(Self::XFERS_POS))
            .expect("expected a transfer list")
            .items()
    }
}

impl<'a, T: Core> Variable<'a, T> {
    pub fn ident(&self) -> atom::Ident {
        self.identifier().ident()
    }

    pub fn identifier(&self) -> Identifier<'a, T> {
        Identifier::cast(self.node().last()).expect("expected an identifier")
    }

    pub fn qualifier(&self) -> Option<Identifier<'a, T>> {
        if self.node().children().count() > 1 {
            let i = Identifier::cast(self.node().first()).expect("expected an identifier");
            Some(i)
        } else {
            None
        }
    }
}

impl<'a, T: Core> SubExpression<'a, T> {
    const INNER_POS: usize = 1;

    pub fn inner(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::INNER_POS)
    }
}

type Context<T> = oal_model::grammar::Context<T, Gram>;
type ParserFn<T> = oal_model::grammar::ParserFn<T, Gram>;
type ParserResult = oal_model::grammar::ParserResult<Gram>;
type TokenOrNode = oal_model::grammar::ParserMatch<Gram>;

pub fn parse_program<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let ns = &mut Vec::new();
    let s = repeat(c, s, ns, &[parse_statement]);
    Ok((s, c.compose_node(SyntaxKind::Program, ns)))
}

pub fn parse_statement<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_import(c, s)
        .or_else(|_| parse_declaration(c, s))
        .or_else(|_| parse_resource(c, s))
}

pub fn parse_import<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_token(c, s, TokenKind::KeywordUse)?;
    let (s, n1) = parse_token(c, s, TokenKind::LiteralString)?;
    let (s, n2) =
        parse_qualifier(c, s).unwrap_or_else(|_| (s, c.compose(SyntaxKind::Qualifier, &[])));
    let (s, n3) = parse_token(c, s, TokenKind::ControlSemicolon)?;
    Ok((s, c.compose(SyntaxKind::Import, &[n0, n1, n2, n3])))
}

pub fn parse_qualifier<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_token(c, s, TokenKind::KeywordAs)?;
    let (s, n1) = parse_identifier(c, s)?;
    Ok((s, c.compose(SyntaxKind::Qualifier, &[n0, n1])))
}

pub fn parse_identifier<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_token(c, s, TokenKind::IdentifierReference)
        .or_else(|_| parse_token(c, s, TokenKind::IdentifierValue))
}

pub fn parse_line_annotations<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let ns = &mut Vec::new();
    let p: ParserFn<T> = |c, s| parse_token(c, s, TokenKind::AnnotationLine);
    let s = repeat(c, s, ns, &[p]);
    Ok((s, c.compose(SyntaxKind::Annotations, ns)))
}

pub fn parse_binding<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n) = parse_token(c, s, TokenKind::IdentifierValue)?;
    Ok((s, c.compose(SyntaxKind::Binding, &[n])))
}

pub fn parse_bindings<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let ns = &mut Vec::new();
    let s = repeat(c, s, ns, &[parse_binding]);
    Ok((s, c.compose(SyntaxKind::Bindings, ns)))
}

pub fn parse_recursion<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_token(c, s, TokenKind::KeywordRec)?;
    let (s, n1) = parse_binding(c, s)?;
    let (s, n2) = parse_expression(c, s)?;
    Ok((s, c.compose(SyntaxKind::Recursion, &[n0, n1, n2])))
}

pub fn parse_xfer_list<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let ns = &mut Vec::new();
    let s = intersperse(c, s, ns, parse_expression, |c, s| {
        parse_token(c, s, TokenKind::ControlComma)
    })?;
    Ok((s, c.compose(SyntaxKind::XferList, ns)))
}

pub fn parse_relation<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_term_kind(c, s)?;
    let (s, n1) = parse_token(c, s, TokenKind::KeywordOn)?;
    let (s, n2) = parse_xfer_list(c, s)?;
    Ok((s, c.compose(SyntaxKind::Relation, &[n0, n1, n2])))
}

pub fn parse_literal<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_token_with(c, s, TokenKind::is_literal)
}

pub fn parse_primitive<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_token_with(c, s, TokenKind::is_primitive)
}

pub fn parse_comma<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_token(c, s, TokenKind::ControlComma)
}

pub fn parse_property_list<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let ns = &mut Vec::new();
    let s = repeat(c, s, ns, &[parse_expression, parse_comma]);
    Ok((s, c.compose(SyntaxKind::PropertyList, ns)))
}

pub fn parse_object<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_token(c, s, TokenKind::ControlBraceLeft)?;
    let (s, n1) = parse_property_list(c, s)?;
    let (s, n2) = parse_token(c, s, TokenKind::ControlBraceRight)?;
    Ok((s, c.compose(SyntaxKind::Object, &[n0, n1, n2])))
}

pub fn parse_uri_root<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_token(c, s, TokenKind::PathElementRoot)
}

pub fn parse_uri_var<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_token(c, s, TokenKind::PathElementRoot)?;
    let (s, n1) = parse_token(c, s, TokenKind::ControlBraceLeft)?;
    let (s, n2) = parse_expression(c, s)?;
    let (s, n3) = parse_token(c, s, TokenKind::ControlBraceRight)?;
    Ok((s, c.compose(SyntaxKind::UriVariable, &[n0, n1, n2, n3])))
}

pub fn parse_uri_segment<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_token(c, s, TokenKind::PathElementSegment)
        .or_else(|_| parse_uri_var(c, s))
        .or_else(|_| parse_uri_root(c, s))
}

pub fn parse_uri_path<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n) = parse_uri_segment(c, s)?;
    let ns = &mut vec![n];
    let s = repeat(c, s, ns, &[parse_uri_segment]);
    Ok((s, c.compose(SyntaxKind::UriPath, ns)))
}

pub fn parse_uri_params<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_token(c, s, TokenKind::OperatorQuestionMark)?;
    let (s, n1) = parse_object(c, s)?;
    Ok((s, c.compose(SyntaxKind::UriParams, &[n0, n1])))
}

pub fn parse_uri_template<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let ns = &mut Vec::new();
    let (s, n) = parse_uri_path(c, s)?;
    ns.push(n);
    let s = if let Ok((s, n)) = parse_uri_params(c, s) {
        ns.push(n);
        s
    } else {
        s
    };
    Ok((s, c.compose(SyntaxKind::UriTemplate, ns)))
}

pub fn parse_uri_kind<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_token(c, s, TokenKind::PrimitiveUri).or_else(|_| parse_uri_template(c, s))
}

pub fn parse_array<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_token(c, s, TokenKind::ControlBracketLeft)?;
    let (s, n1) = parse_expression(c, s)?;
    let (s, n2) = parse_token(c, s, TokenKind::ControlBracketRight)?;
    Ok((s, c.compose(SyntaxKind::Array, &[n0, n1, n2])))
}

pub fn parse_property<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let ns = &mut Vec::new();
    let (s, n) = parse_token(c, s, TokenKind::Property)?;
    ns.push(n);
    let s = if let Ok((s, n)) = parse_token(c, s, TokenKind::OperatorExclamationMark)
        .or_else(|_| parse_token(c, s, TokenKind::OperatorQuestionMark))
    {
        ns.push(n);
        s
    } else {
        s
    };
    let (s, n) = parse_expression(c, s)?;
    ns.push(n);
    Ok((s, c.compose(SyntaxKind::Property, ns)))
}

pub fn parse_content_meta<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_token_with(c, s, TokenKind::is_content)?;
    let (s, n1) = parse_token(c, s, TokenKind::OperatorEqual)?;
    let (s, n2) = parse_expression(c, s)?;
    Ok((s, c.compose(SyntaxKind::ContentMeta, &[n0, n1, n2])))
}

pub fn parse_content_meta_list<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let ns = &mut Vec::new();
    let s = intersperse(c, s, ns, parse_content_meta, parse_comma)?;
    Ok((s, c.compose(SyntaxKind::ContentMetaList, ns)))
}

pub fn parse_content_body<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n) = parse_expression(c, s)?;
    Ok((s, c.compose(SyntaxKind::ContentBody, &[n])))
}

pub fn parse_content_1<T: Core>(
    c: &mut Context<T>,
    s: Cursor,
    ns: &mut Vec<TokenOrNode>,
) -> std::result::Result<Cursor, ParserError> {
    let (s, n0) = parse_content_meta_list(c, s)?;
    let (s, n1) = parse_comma(c, s)?;
    let (s, n2) = parse_content_body(c, s)?;
    ns.push(n0);
    ns.push(n1);
    ns.push(n2);
    Ok(s)
}

pub fn parse_content_2<T: Core>(
    c: &mut Context<T>,
    s: Cursor,
    ns: &mut Vec<TokenOrNode>,
) -> std::result::Result<Cursor, ParserError> {
    let (s, n) = parse_content_meta_list(c, s)?;
    ns.push(n);
    Ok(s)
}

pub fn parse_content_3<T: Core>(
    c: &mut Context<T>,
    s: Cursor,
    ns: &mut Vec<TokenOrNode>,
) -> std::result::Result<Cursor, ParserError> {
    let (s, n) = parse_content_body(c, s)?;
    ns.push(n);
    Ok(s)
}

pub fn parse_content<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let ns = &mut Vec::new();
    let (s, n) = parse_token(c, s, TokenKind::ControlChevronLeft)?;
    ns.push(n);
    let s = parse_content_1(c, s, ns)
        .or_else(|_| parse_content_2(c, s, ns))
        .or_else(|_| parse_content_3(c, s, ns))
        .unwrap_or(s);
    let (s, n) = parse_token(c, s, TokenKind::ControlChevronRight)?;
    ns.push(n);
    Ok((s, c.compose(SyntaxKind::Content, ns)))
}

#[test]
fn test_parse_content() {
    test_parser::<()>(
        parse_content,
        vec![
            TokenKind::ControlChevronLeft,
            TokenKind::ContentMedia,
            TokenKind::OperatorEqual,
            TokenKind::LiteralString,
            TokenKind::ControlComma,
            TokenKind::ContentStatus,
            TokenKind::OperatorEqual,
            TokenKind::LiteralNumber,
            TokenKind::ControlComma,
            TokenKind::ContentHeaders,
            TokenKind::OperatorEqual,
            TokenKind::ControlBraceLeft,
            TokenKind::ControlBraceRight,
            TokenKind::ControlComma,
            TokenKind::ControlBraceLeft,
            TokenKind::ControlBraceRight,
            TokenKind::ControlChevronRight,
        ],
    );
}

pub fn parse_subexpr<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_token(c, s, TokenKind::ControlParenLeft)?;
    let (s, n1) = parse_expression(c, s)?;
    let (s, n2) = parse_token(c, s, TokenKind::ControlParenRight)?;
    Ok((s, c.compose(SyntaxKind::SubExpression, &[n0, n1, n2])))
}

pub fn parse_term<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_line_annotations(c, s)?;
    let (s, n1) = parse_literal(c, s)
        .or_else(|_| parse_primitive(c, s))
        .or_else(|_| parse_uri_kind(c, s))
        .or_else(|_| parse_array(c, s))
        .or_else(|_| parse_property(c, s))
        .or_else(|_| parse_object(c, s))
        .or_else(|_| parse_content(c, s))
        .or_else(|_| parse_subexpr(c, s))
        .or_else(|_| parse_variable(c, s))?;
    let ns = &mut vec![n0, n1];
    let s = if let Ok((s, n2)) = parse_token(c, s, TokenKind::AnnotationInline) {
        ns.push(n2);
        s
    } else {
        s
    };
    Ok((s, c.compose(SyntaxKind::Terminal, ns)))
}

pub fn parse_term_kind<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    memoize(ParserTag::Term, c, s, parse_term)
}

pub fn parse_optional_kind<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_term_kind(c, s)?;
    let (s, n1) = parse_token(c, s, TokenKind::OperatorQuestionMark)?;
    Ok((s, c.compose(SyntaxKind::UnaryOp, &[n0, n1])))
}

pub fn parse_required_kind<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_term_kind(c, s)?;
    let (s, n1) = parse_token(c, s, TokenKind::OperatorExclamationMark)?;
    Ok((s, c.compose(SyntaxKind::UnaryOp, &[n0, n1])))
}

pub fn parse_unary_kind<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_optional_kind(c, s)
        .or_else(|_| parse_required_kind(c, s))
        .or_else(|_| parse_term_kind(c, s))
}

pub fn parse_variable<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let ns = &mut Vec::new();
    let (s, n0) = parse_identifier(c, s)?;
    ns.push(n0);
    let s = if let Ok((s, n1)) = parse_token(c, s, TokenKind::ControlFullStop) {
        let (s, n2) = parse_identifier(c, s)?;
        ns.push(n1);
        ns.push(n2);
        s
    } else {
        s
    };
    Ok((s, c.compose(SyntaxKind::Variable, ns)))
}

pub fn parse_application<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let ns = &mut Vec::new();
    let (s, n0) = parse_variable(c, s)?;
    ns.push(n0);
    let (s, n1) = parse_unary_kind(c, s)?;
    ns.push(n1);
    let s = repeat(c, s, ns, &[parse_unary_kind]);
    Ok((s, c.compose(SyntaxKind::Application, ns)))
}

pub fn parse_apply_kind<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_application(c, s).or_else(|_| parse_unary_kind(c, s))
}

pub fn parse_variadic_op<T: Core>(
    c: &mut Context<T>,
    s: Cursor,
    op: TokenKind,
    p: ParserFn<T>,
) -> ParserResult {
    let ns = &mut Vec::new();
    let s = intersperse(c, s, ns, p, |c, s| parse_token(c, s, op))?;
    let n = if ns.len() == 1 {
        ns.pop().unwrap()
    } else {
        c.compose(SyntaxKind::VariadicOp, ns)
    };
    Ok((s, n))
}

pub fn parse_range_kind<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_variadic_op(c, s, TokenKind::OperatorDoubleColon, parse_apply_kind)
}

pub fn parse_join_kind<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_variadic_op(c, s, TokenKind::OperatorAmpersand, parse_range_kind)
}

pub fn parse_any_kind<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_variadic_op(c, s, TokenKind::OperatorTilde, parse_join_kind)
}

pub fn parse_sum_kind<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_variadic_op(c, s, TokenKind::OperatorVerticalBar, parse_any_kind)
}

pub fn parse_xfer_domain<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_token(c, s, TokenKind::OperatorColon)?;
    let (s, n1) = parse_term_kind(c, s)?;
    Ok((s, c.compose(SyntaxKind::XferDomain, &[n0, n1])))
}

pub fn parse_xfer_params<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n) = parse_object(c, s)?;
    Ok((s, c.compose(SyntaxKind::XferParams, &[n])))
}

pub fn parse_xfer_methods<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let ns = &mut Vec::new();
    let method: ParserFn<T> = |c, s| parse_token_with(c, s, TokenKind::is_method);
    let s = intersperse(c, s, ns, method, parse_comma)?;
    Ok((s, c.compose(SyntaxKind::XferMethods, ns)))
}

#[test]
fn test_parse_xfer_methods() {
    test_parser::<()>(
        parse_xfer_methods,
        vec![
            TokenKind::MethodGet,
            TokenKind::ControlComma,
            TokenKind::MethodPut,
        ],
    );
}

pub fn parse_transfer<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_xfer_methods(c, s)?;
    let (s, n1) =
        parse_xfer_params(c, s).unwrap_or_else(|_| (s, c.compose(SyntaxKind::XferParams, &[])));
    let (s, n2) =
        parse_xfer_domain(c, s).unwrap_or_else(|_| (s, c.compose(SyntaxKind::XferDomain, &[])));
    let (s, n3) = parse_token(c, s, TokenKind::OperatorArrow)?;
    let (s, n4) = parse_range_kind(c, s)?;
    Ok((s, c.compose(SyntaxKind::Transfer, &[n0, n1, n2, n3, n4])))
}

#[test]
fn test_parse_transfer() {
    test_parser::<()>(
        parse_transfer,
        vec![
            TokenKind::MethodGet,
            TokenKind::ControlComma,
            TokenKind::MethodPut,
            TokenKind::OperatorArrow,
            TokenKind::ControlChevronLeft,
            TokenKind::ControlChevronRight,
        ],
    );
}

pub fn parse_xfer_kind<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_transfer(c, s).or_else(|_| parse_sum_kind(c, s))
}

pub fn parse_relation_kind<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    parse_relation(c, s).or_else(|_| parse_xfer_kind(c, s))
}

pub fn parse_expression<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    memoize(ParserTag::Expression, c, s, |c, s| {
        parse_recursion(c, s).or_else(|_| parse_relation_kind(c, s))
    })
}

pub fn parse_declaration<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_line_annotations(c, s)?;
    let (s, n1) = parse_token(c, s, TokenKind::KeywordLet)?;
    let (s, n2) = parse_identifier(c, s)?;
    let (s, n3) = parse_bindings(c, s)?;
    match (&n2, &n3) {
        (ParserMatch::Token(t), ParserMatch::Node(_))
            if t.kind() == TokenKind::IdentifierReference =>
        {
            return Err(ParserError::new(
                "invalid reference identifier (function)",
                c.span(s),
            ))
        }
        _ => {}
    }
    let (s, n4) = parse_token(c, s, TokenKind::OperatorEqual)?;
    let (s, n5) = parse_expression(c, s)?;
    let (s, n6) = parse_token(c, s, TokenKind::ControlSemicolon)?;
    Ok((
        s,
        c.compose(SyntaxKind::Declaration, &[n0, n1, n2, n3, n4, n5, n6]),
    ))
}

pub fn parse_resource<T: Core>(c: &mut Context<T>, s: Cursor) -> ParserResult {
    let (s, n0) = parse_token(c, s, TokenKind::KeywordRes)?;
    let (s, n1) = parse_expression(c, s)?;
    let (s, n2) = parse_token(c, s, TokenKind::ControlSemicolon)?;
    Ok((s, c.compose(SyntaxKind::Resource, &[n0, n1, n2])))
}

#[cfg(test)]
fn test_parser<T: Core>(parser: ParserFn<T>, tokens: Vec<TokenKind>) {
    let loc = Locator::try_from("file:///example.oal").unwrap();
    let mut l = TokenList::new(loc.clone());
    for (i, k) in tokens.iter().enumerate() {
        let t = Token::new(*k, TokenValue::None);
        l.push(t, i..i + 1);
    }
    let mut ctx = Context::new(l);
    let cursor = ctx.head();
    let (end, root) = parser(&mut ctx, cursor).unwrap();
    assert!(!end.is_valid(), "remaining input");
    let ParserMatch::Node(root) = root else {
        panic!("root should be a node")
    };
    println!("{:?}", ctx);
    let count = ctx.count();
    let tree = ctx.tree().finalize(root);
    assert_eq!(tree.root().descendants().count(), count, "syntax tree leak");
}

#[test]
fn test_misc() {
    let cases = [
        vec![
            TokenKind::KeywordUse,
            TokenKind::LiteralString,
            TokenKind::KeywordAs,
            TokenKind::IdentifierValue,
            TokenKind::ControlSemicolon,
        ],
        vec![
            TokenKind::KeywordLet,
            TokenKind::IdentifierValue,
            TokenKind::OperatorEqual,
            TokenKind::PrimitiveNum,
            TokenKind::ControlSemicolon,
        ],
        vec![
            TokenKind::KeywordLet,
            TokenKind::IdentifierReference,
            TokenKind::OperatorEqual,
            TokenKind::ControlBraceLeft,
            TokenKind::ControlBraceRight,
            TokenKind::ControlSemicolon,
        ],
        vec![
            TokenKind::KeywordLet,
            TokenKind::IdentifierReference,
            TokenKind::OperatorEqual,
            TokenKind::ControlBraceLeft,
            TokenKind::Property,
            TokenKind::ControlBraceLeft,
            TokenKind::Property,
            TokenKind::ControlBraceLeft,
            TokenKind::Property,
            TokenKind::ControlBraceLeft,
            TokenKind::Property,
            TokenKind::ControlBraceLeft,
            TokenKind::Property,
            TokenKind::PrimitiveNum,
            TokenKind::ControlBraceRight,
            TokenKind::ControlBraceRight,
            TokenKind::ControlBraceRight,
            TokenKind::ControlBraceRight,
            TokenKind::ControlBraceRight,
            TokenKind::ControlSemicolon,
        ],
        vec![
            TokenKind::KeywordRes,
            TokenKind::PathElementRoot,
            TokenKind::KeywordOn,
            TokenKind::MethodGet,
            TokenKind::OperatorArrow,
            TokenKind::ControlChevronLeft,
            TokenKind::ControlChevronRight,
            TokenKind::ControlSemicolon,
        ],
    ];

    for tokens in cases {
        test_parser::<()>(parse_program, tokens);
    }
}
