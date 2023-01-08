use crate::atom;
use crate::lexer as lex;
use crate::lexer::{Token, TokenKind, TokenValue};
use chumsky::prelude::*;
use oal_model::grammar::*;
use oal_model::lexicon::TokenAlias;
use oal_model::{match_token, syntax_nodes, terminal_node};
use std::fmt::Debug;

#[derive(Copy, Clone, Default, Debug)]
pub struct Gram;

impl Grammar for Gram {
    type Lex = Token;
    type Kind = SyntaxKind;
}

terminal_node!(Gram, Identifier, TokenKind::Identifier(_));

impl<'a, T: Core> Identifier<'a, T> {
    pub fn ident(&self) -> atom::Ident {
        self.node().as_str().into()
    }
}

terminal_node!(
    Gram,
    Primitive,
    TokenKind::Keyword(lex::Keyword::Primitive(_))
);

impl<'a, T: Core> Primitive<'a, T> {
    pub fn primitive(&self) -> lex::Primitive {
        let TokenKind::Keyword(lex::Keyword::Primitive(p)) = self.node().token().kind() else { unreachable!() };
        p
    }
}

terminal_node!(Gram, PathElement, TokenKind::PathElement(_));

impl<'a, T: Core> PathElement<'a, T> {
    pub fn as_str(&self) -> &'a str {
        match self.node().token().kind() {
            // Note that path separators are omitted from the string representation.
            TokenKind::PathElement(lex::PathElement::Root) => "",
            TokenKind::PathElement(lex::PathElement::Segment) => self.node().as_str(),
            _ => unreachable!(),
        }
    }
}

terminal_node!(Gram, PropertyName, TokenKind::Property);

impl<'a, T: Core> PropertyName<'a, T> {
    pub fn as_ident(&self) -> atom::Ident {
        self.node().as_str().into()
    }
}

terminal_node!(Gram, Method, TokenKind::Keyword(lex::Keyword::Method(_)));

impl<'a, T: Core> Method<'a, T> {
    pub fn method(&self) -> atom::Method {
        let TokenKind::Keyword(lex::Keyword::Method(m)) = self.node().token().kind() else { unreachable!() };
        m
    }
}

terminal_node!(Gram, Literal, TokenKind::Literal(_));

impl<'a, T: Core> Literal<'a, T> {
    pub fn kind(&self) -> lex::Literal {
        let TokenKind::Literal(l) = self.node().token().kind() else { unreachable!() };
        l
    }

    pub fn value(&self) -> &'a TokenValue {
        self.node().token().value()
    }

    pub fn as_str(&self) -> &'a str {
        self.node().as_str()
    }
}

terminal_node!(
    Gram,
    ContentTag,
    TokenKind::Keyword(lex::Keyword::Content(_))
);

impl<'a, T: Core> ContentTag<'a, T> {
    pub fn tag(&self) -> lex::Content {
        let TokenKind::Keyword(lex::Keyword::Content(t)) = self.node().token().kind() else { unreachable!() };
        t
    }
}

terminal_node!(Gram, Operator, TokenKind::Operator(_));

impl<'a, T: Core> Operator<'a, T> {
    pub fn operator(&self) -> atom::Operator {
        let TokenKind::Operator(op) = self.node().token().kind() else { unreachable!() };
        match op {
            lex::Operator::DoubleColon => atom::Operator::Range,
            lex::Operator::Ampersand => atom::Operator::Join,
            lex::Operator::Tilde => atom::Operator::Any,
            lex::Operator::VerticalBar => atom::Operator::Sum,
            _ => unreachable!("not a variadic operator {:?}", op),
        }
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
    XferMethods,
    XferParams,
    XferDomain,
    Transfer,
    Import,
    Resource,
    XferList,
    Relation,
    Program
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
    pub fn items(&self) -> impl Iterator<Item = &'a str> {
        self.node().children().map(|c| c.as_str())
    }
}

impl<'a, T: Core> Binding<'a, T> {
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

    pub fn annotations(&self) -> impl Iterator<Item = &'a str> {
        Annotations::cast(self.node().nth(Self::ANNOTATIONS_POS))
            .expect("expected annotations")
            .items()
    }

    pub fn ident(&self) -> atom::Ident {
        Identifier::cast(self.node().nth(Self::IDENTIFIER_POS))
            .expect("declaration lhs must be an identifier")
            .ident()
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

impl<'a, T: Core> Import<'a, T> {
    const MODULE_POS: usize = 1;

    pub fn module(&self) -> &'a str {
        self.node().nth(Self::MODULE_POS).as_str()
    }
}

impl<'a, T: Core> Terminal<'a, T> {
    const PREFIX_ANN_POS: usize = 0;
    const INNER_POS: usize = 1;
    const SUFFIX_ANN_POS: usize = 2;

    pub fn inner(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::INNER_POS)
    }

    pub fn prefix_annotations(&self) -> impl Iterator<Item = &'a str> {
        Annotations::cast(self.node().nth(Self::PREFIX_ANN_POS))
            .expect("expected annotations")
            .items()
    }

    pub fn suffix_annotation(&self) -> Option<&'a str> {
        self.node()
            .children()
            .nth(Self::SUFFIX_ANN_POS)
            .map(|n| n.as_str())
    }

    pub fn annotations(&self) -> impl Iterator<Item = &'a str> {
        self.prefix_annotations()
            .chain(self.suffix_annotation().into_iter())
    }
}

impl<'a, T: Core> Array<'a, T> {
    const INNER_POS: usize = 1;

    pub fn inner(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::INNER_POS)
    }
}

impl<'a, T: Core> UriVariable<'a, T> {
    const INNER_POS: usize = 1;

    pub fn inner(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::INNER_POS)
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
    const NAME_POS: usize = 0;
    const RHS_POS: usize = 1;

    pub fn name(&self) -> atom::Ident {
        PropertyName::cast(self.node().nth(Self::NAME_POS))
            .expect("expected a property name")
            .as_ident()
    }

    pub fn rhs(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::RHS_POS)
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

    pub fn operator(&self) -> atom::Operator {
        Operator::cast(self.node().nth(Self::OPERATOR_POS))
            .expect("expected an operator")
            .operator()
    }

    pub fn operands(&self) -> impl Iterator<Item = NodeRef<'a, T, Gram>> {
        self.node().children().step_by(2)
    }
}

impl<'a, T: Core> ContentMeta<'a, T> {
    const TAG_POS: usize = 0;
    const RHS_POS: usize = 2;

    pub fn tag(&self) -> lex::Content {
        ContentTag::cast(self.node().nth(Self::TAG_POS))
            .expect("expected content tag")
            .tag()
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
    const INNER_POS: usize = 0;

    pub fn ident(&self) -> atom::Ident {
        Identifier::cast(self.node().nth(Self::INNER_POS))
            .expect("expected an identifier")
            .ident()
    }
}

impl<'a, T: Core> SubExpression<'a, T> {
    const INNER_POS: usize = 1;

    pub fn inner(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::INNER_POS)
    }
}

fn variadic_op<'a, P, E>(
    op: lex::Operator,
    p: P,
) -> impl Parser<TokenAlias<Token>, ParseNode<Gram>, Error = E> + Clone + 'a
where
    P: Parser<TokenAlias<Token>, ParseNode<Gram>, Error = E> + Clone + 'a,
    E: chumsky::Error<TokenAlias<Token>> + 'a,
{
    tree_skip(
        p.clone().chain(
            just_token(TokenKind::Operator(op))
                .chain(p)
                .repeated()
                .flatten(),
        ),
        SyntaxKind::VariadicOp,
    )
}

pub fn parser<'a>(
) -> impl Parser<TokenAlias<Token>, ParseNode<Gram>, Error = ParserError<Token>> + 'a {
    let identifier = match_token! { TokenKind::Identifier(_) };

    let literal = match_token! { TokenKind::Literal(_) };

    let primitive = match_token! { TokenKind::Keyword(lex::Keyword::Primitive(_)) };

    let uri_root = just_token(TokenKind::PathElement(lex::PathElement::Root));

    let uri_segment = just_token(TokenKind::PathElement(lex::PathElement::Segment));

    let method = match_token! { TokenKind::Keyword(lex::Keyword::Method(_)) };

    let xfer_methods = tree_many(
        method.chain(
            just_token(TokenKind::Control(lex::Control::Comma))
                .chain(method)
                .repeated()
                .flatten(),
        ),
        SyntaxKind::XferMethods,
    );

    let inline_annotation = just_token(TokenKind::Annotation(lex::Annotation::Inline));

    let line_annotations = || {
        tree_many(
            just_token(TokenKind::Annotation(lex::Annotation::Line)).repeated(),
            SyntaxKind::Annotations,
        )
    };

    let expr_kind = recursive(|expr| {
        let property_list = tree_many(
            expr.clone()
                .chain(
                    just_token(TokenKind::Control(lex::Control::Comma))
                        .chain(expr.clone())
                        .repeated()
                        .flatten(),
                )
                .or_not()
                .flatten(),
            SyntaxKind::PropertyList,
        );

        let object = tree_many(
            just_token(TokenKind::Control(lex::Control::BraceLeft))
                .chain(property_list)
                .chain(just_token(TokenKind::Control(lex::Control::BraceRight))),
            SyntaxKind::Object,
        );

        let uri_var = tree_many(
            just_token(TokenKind::Control(lex::Control::BraceLeft))
                .chain(expr.clone())
                .chain(just_token(TokenKind::Control(lex::Control::BraceRight))),
            SyntaxKind::UriVariable,
        );

        let uri_path = tree_many(
            uri_segment
                .or(uri_root.clone().ignore_then(uri_var))
                .or(uri_root)
                .repeated()
                .at_least(1)
                .collect(),
            SyntaxKind::UriPath,
        );

        let uri_params = tree_many(
            just_token(TokenKind::Operator(lex::Operator::QuestionMark)).chain(object.clone()),
            SyntaxKind::UriParams,
        );

        let uri_template = tree_many(uri_path.chain(uri_params.or_not()), SyntaxKind::UriTemplate);

        let uri_kind = just_token(TokenKind::Keyword(lex::Keyword::Primitive(
            lex::Primitive::Uri,
        )))
        .or(uri_template);

        let array = tree_many(
            just_token(TokenKind::Control(lex::Control::BracketLeft))
                .chain(expr.clone())
                .chain(just_token(TokenKind::Control(lex::Control::BracketRight))),
            SyntaxKind::Array,
        );

        let property = tree_many(
            just_token(TokenKind::Property).chain(expr.clone()),
            SyntaxKind::Property,
        );

        let content_meta = tree_many(
            match_token! { TokenKind::Keyword(lex::Keyword::Content(_)) }
                .chain(just_token(TokenKind::Operator(lex::Operator::Equal)))
                .chain(expr.clone()),
            SyntaxKind::ContentMeta,
        );

        let content_meta_list = tree_many(
            content_meta.clone().chain(
                just_token(TokenKind::Control(lex::Control::Comma))
                    .chain(content_meta)
                    .repeated()
                    .flatten(),
            ),
            SyntaxKind::ContentMetaList,
        );

        let content_body = tree_one(expr.clone(), SyntaxKind::ContentBody);

        let content = tree_many(
            just_token(TokenKind::Control(lex::Control::ChevronLeft))
                .chain(
                    content_meta_list
                        .clone()
                        .chain(just_token(TokenKind::Control(lex::Control::Comma)))
                        .chain(content_body.clone())
                        .or(content_meta_list.map(move |n| vec![n]))
                        .or(content_body.map(move |n| vec![n]))
                        .or_not()
                        .flatten(),
                )
                .chain(just_token(TokenKind::Control(lex::Control::ChevronRight))),
            SyntaxKind::Content,
        );

        let subexpr = tree_many(
            just_token(TokenKind::Control(lex::Control::ParenLeft))
                .chain(expr.clone())
                .chain(just_token(TokenKind::Control(lex::Control::ParenRight))),
            SyntaxKind::SubExpression,
        );

        let variable = tree_one(identifier, SyntaxKind::Variable);

        let term_kind = tree_many(
            line_annotations()
                .chain(
                    literal
                        .or(primitive)
                        .or(uri_kind)
                        .or(array)
                        .or(property)
                        .or(object.clone())
                        .or(content)
                        .or(subexpr)
                        .or(variable.clone()),
                )
                .chain(inline_annotation.or_not()),
            SyntaxKind::Terminal,
        );

        let application = tree_many(
            variable.chain(term_kind.clone().repeated().at_least(1)),
            SyntaxKind::Application,
        );

        let apply_kind = application.or(term_kind.clone());

        let range_kind = variadic_op(lex::Operator::DoubleColon, apply_kind);

        let join_kind = variadic_op(lex::Operator::Ampersand, range_kind.clone());

        let any_kind = variadic_op(lex::Operator::Tilde, join_kind);

        let sum_kind = variadic_op(lex::Operator::VerticalBar, any_kind);

        let xfer_params = tree_maybe(object.or_not(), SyntaxKind::XferParams);

        let xfer_domain = tree_many(
            just_token(TokenKind::Operator(lex::Operator::Colon))
                .chain(term_kind.clone())
                .or_not()
                .flatten(),
            SyntaxKind::XferDomain,
        );

        let transfer = tree_many(
            xfer_methods
                .chain(xfer_params)
                .chain(xfer_domain)
                .chain(just_token(TokenKind::Operator(lex::Operator::Arrow)))
                .chain(range_kind),
            SyntaxKind::Transfer,
        );

        let xfer_kind = transfer.or(sum_kind);

        let xfer_list = tree_many(
            xfer_kind.clone().chain(
                just_token(TokenKind::Control(lex::Control::Comma))
                    .chain(xfer_kind.clone())
                    .repeated()
                    .flatten(),
            ),
            SyntaxKind::XferList,
        );

        let relation = tree_many(
            term_kind
                .chain(just_token(TokenKind::Control(lex::Control::ParenLeft)))
                .chain(xfer_list)
                .chain(just_token(TokenKind::Control(lex::Control::ParenRight))),
            SyntaxKind::Relation,
        );

        relation.or(xfer_kind)
    });

    let binding = tree_one(
        just_token(TokenKind::Identifier(lex::Identifier::Value)),
        SyntaxKind::Binding,
    );

    let bindings = tree_many(binding.repeated(), SyntaxKind::Bindings);

    let declaration = tree_many(
        line_annotations()
            .chain(just_token(TokenKind::Keyword(lex::Keyword::Let)))
            .chain(identifier)
            .chain(bindings)
            .chain(just_token(TokenKind::Operator(lex::Operator::Equal)))
            .chain(expr_kind.clone())
            .chain(just_token(TokenKind::Control(lex::Control::Semicolon))),
        SyntaxKind::Declaration,
    );

    let resource = tree_many(
        just_token(TokenKind::Keyword(lex::Keyword::Res))
            .chain(expr_kind)
            .chain(just_token(TokenKind::Control(lex::Control::Semicolon))),
        SyntaxKind::Resource,
    );

    let import = tree_many(
        just_token(TokenKind::Keyword(lex::Keyword::Use))
            .chain(just_token(TokenKind::Literal(lex::Literal::String)))
            .chain(just_token(TokenKind::Control(lex::Control::Semicolon))),
        SyntaxKind::Import,
    );

    // TODO: implement skip_until for composite input types or refactor TokenAlias into a scalar.
    let statement = declaration
        .or(resource)
        .or(import)
        .recover_with(skip_until([], |_| ParseNode::from_error()));

    tree_many(statement.repeated(), SyntaxKind::Program)
}
