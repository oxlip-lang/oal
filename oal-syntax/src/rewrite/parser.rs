use crate::atom::Ident;
use crate::rewrite::lexer as lex;
use crate::rewrite::lexer::{Token, TokenKind, TokenValue};
use chumsky::prelude::*;
use oal_model::grammar::*;
use oal_model::lexicon::{Interner, TokenAlias};
use oal_model::{syntax_nodes, terminal_node};
use std::fmt::Debug;

#[derive(Copy, Clone, Default, Debug)]
pub struct Gram;

impl Grammar for Gram {
    type Lex = Token;
    type Kind = SyntaxKind;
}

terminal_node!(Gram, Symbol, TokenKind::Identifier(_));

impl<'a, T: Default + Clone> Symbol<'a, T> {
    pub fn as_ident(&self) -> Ident {
        match self.node().token().value() {
            TokenValue::Symbol(sym) => self.node().tree().resolve(*sym).into(),
            _ => panic!("identifier must be a registered string"),
        }
    }
}

terminal_node!(
    Gram,
    Primitive,
    TokenKind::Keyword(lex::Keyword::Primitive(_))
);

impl<'a, T: Default + Clone> Primitive<'a, T> {
    pub fn primitive(&self) -> lex::Primitive {
        let TokenKind::Keyword(lex::Keyword::Primitive(p)) = self.node().token().kind() else { unreachable!() };
        p
    }
}

terminal_node!(Gram, PathElement, TokenKind::PathElement(_));

impl<'a, T: Default + Clone> PathElement<'a, T> {
    pub fn as_str(&self) -> &str {
        match self.node().token().kind() {
            TokenKind::PathElement(lex::PathElement::Root) => "/",
            TokenKind::PathElement(lex::PathElement::Segment) => {
                if let TokenValue::Symbol(sym) = self.node().token().value() {
                    self.node().tree().resolve(*sym)
                } else {
                    panic!("path segment must be a registered string")
                }
            }
            _ => unreachable!(),
        }
    }
}

terminal_node!(Gram, ProperyName, TokenKind::Property);

impl<'a, T: Default + Clone> ProperyName<'a, T> {
    pub fn as_ident(&self) -> Ident {
        match self.node().token().value() {
            TokenValue::Symbol(sym) => self.node().tree().resolve(*sym).into(),
            _ => panic!("property name must be a registered string"),
        }
    }
}

syntax_nodes!(
    Gram,
    Terminal,
    SubExpression,
    Content,
    Property,
    Array,
    Declaration,
    UriVariable,
    UriPath,
    UriParams,
    UriTemplate,
    Object,
    Application,
    VariadicOp,
    Transfer,
    Import,
    Resource,
    Relation,
    Program
);

impl<'a, T: Default + Clone> Program<'a, T> {
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

impl<'a, T: Default + Clone> Declaration<'a, T> {
    const SYM_POS: usize = 1;
    const RHS_POS: usize = 3;

    pub fn symbol(&self) -> Symbol<'a, T> {
        let Some(symbol) = Symbol::cast(self.node().nth(Self::SYM_POS)) else { panic!("declaration lhs must be a symbol") };
        symbol
    }

    pub fn rhs(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::RHS_POS)
    }
}

impl<'a, T: Default + Clone> Import<'a, T> {
    const MODULE_POS: usize = 1;

    pub fn module(&self) -> &'a str {
        match self.node().nth(Self::MODULE_POS).token().value() {
            TokenValue::Symbol(sym) => self.node().tree().resolve(*sym),
            _ => panic!("module must be a symbol"),
        }
    }
}

impl<'a, T: Default + Clone> Terminal<'a, T> {
    pub fn inner(&self) -> NodeRef<'a, T, Gram> {
        self.node().first()
    }
}

impl<'a, T: Default + Clone> Array<'a, T> {
    const INNER_POS: usize = 1;

    pub fn inner(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::INNER_POS)
    }
}

impl<'a, T: Default + Clone> UriTemplate<'a, T> {
    const PATH_POS: usize = 0;
    const PARAMS_POS: usize = 1;

    pub fn path(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::PATH_POS)
    }

    pub fn params(&self) -> Option<UriParams<'a, T>> {
        self.node()
            .children()
            .nth(Self::PARAMS_POS)
            .and_then(UriParams::cast)
    }
}

impl<'a, T: Default + Clone> UriVariable<'a, T> {
    const INNER_POS: usize = 1;

    pub fn inner(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::INNER_POS)
    }
}

impl<'a, T: Default + Clone> UriParams<'a, T> {
    const INNER_POS: usize = 1;

    pub fn properties(&self) -> impl Iterator<Item = NodeRef<'a, T, Gram>> {
        let Some(object) = Object::cast(self.node().nth(Self::INNER_POS)) else { panic!("URI parameters must be an object") };
        object.properties()
    }
}

impl<'a, T: Default + Clone> Property<'a, T> {
    const NAME_POS: usize = 0;
    const RHS_POS: usize = 1;

    pub fn name(&self) -> ProperyName<'a, T> {
        let Some(name) = ProperyName::cast(self.node().nth(Self::NAME_POS)) else { panic!("expected a propery name") };
        name
    }

    pub fn rhs(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::RHS_POS)
    }
}

impl<'a, T: Default + Clone> Object<'a, T> {
    pub fn properties(&self) -> impl Iterator<Item = NodeRef<'a, T, Gram>> {
        self.node().children().skip(1).step_by(2)
    }
}

#[derive(Debug)]
pub enum UriSegment<'a, T: Clone + Default> {
    Element(PathElement<'a, T>),
    Variable(UriVariable<'a, T>),
}

impl<'a, T: Default + Clone> UriPath<'a, T> {
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

fn just_<E>(kind: TokenKind) -> impl Parser<TokenAlias<Token>, TokenAlias<Token>, Error = E> + Clone
where
    E: chumsky::Error<TokenAlias<Token>>,
{
    just_token::<_, Gram>(kind)
}

fn variadic_op<'a, P, E, T>(
    op: lex::Operator,
    p: P,
) -> impl Parser<TokenAlias<Token>, ParseNode<T, Gram>, Error = E> + Clone + 'a
where
    P: Parser<TokenAlias<Token>, ParseNode<T, Gram>, Error = E> + Clone + 'a,
    E: chumsky::Error<TokenAlias<Token>> + 'a,
    T: Default + Clone + 'a,
{
    p.clone()
        .chain(
            just_(TokenKind::Operator(op))
                .leaf()
                .chain(p)
                .repeated()
                .flatten(),
        )
        .skip_tree(SyntaxKind::VariadicOp)
}

macro_rules! match_ {
    ($($p:pat $(if $guard:expr)?),+ $(,)?) => ({
        chumsky::primitive::filter_map(move |span, x: TokenAlias<Token>| match x.kind() {
            $($p $(if $guard)? => ::core::result::Result::Ok(x)),+,
            _ => ::core::result::Result::Err(
                chumsky::error::Error::expected_input_found(
                    span, ::core::option::Option::None, ::core::option::Option::Some(x)
                )
            ),
        })
    });
}

pub fn parser<'a, T>(
) -> impl Parser<TokenAlias<Token>, ParseNode<T, Gram>, Error = Simple<TokenAlias<Token>>> + 'a
where
    T: Default + Clone + 'a,
{
    let binding = just_(TokenKind::Identifier(lex::Identifier::Value)).leaf();

    let variable = match_! { TokenKind::Identifier(_) }.leaf();

    let literal_type = match_! { TokenKind::Literal(_) }.leaf();

    let prim_type = match_! { TokenKind::Keyword(lex::Keyword::Primitive(_)) }.leaf();

    let uri_root = just_(TokenKind::PathElement(lex::PathElement::Root)).leaf();

    let uri_segment = just_(TokenKind::PathElement(lex::PathElement::Segment)).leaf();

    let method = match_! { TokenKind::Keyword(lex::Keyword::Method(_)) }.leaf();

    let methods = method.chain(
        just_(TokenKind::Control(lex::Control::Comma))
            .leaf()
            .chain(method)
            .repeated()
            .flatten(),
    );

    let line_ann = just_(TokenKind::Annotation(lex::Annotation::Line)).leaf();

    let inline_ann = just_(TokenKind::Annotation(lex::Annotation::Inline)).leaf();

    let expr_type = recursive(|expr| {
        let object_type = just_(TokenKind::Control(lex::Control::BlockBegin))
            .leaf()
            .chain(
                expr.clone()
                    .chain(
                        just_(TokenKind::Control(lex::Control::Comma))
                            .leaf()
                            .chain(expr.clone())
                            .repeated()
                            .flatten(),
                    )
                    .or_not()
                    .flatten(),
            )
            .chain(just_(TokenKind::Control(lex::Control::BlockEnd)).leaf())
            .tree(SyntaxKind::Object);

        let uri_var = just_(TokenKind::Control(lex::Control::BlockBegin))
            .leaf()
            .chain(expr.clone())
            .chain(just_(TokenKind::Control(lex::Control::BlockEnd)).leaf())
            .tree(SyntaxKind::UriVariable);

        let uri_path = uri_segment
            .map(|l| vec![l])
            .or(uri_root.chain(uri_var.or_not()))
            .repeated()
            .at_least(1)
            .flatten()
            .collect()
            .tree(SyntaxKind::UriPath);

        let uri_params = just_(TokenKind::Operator(lex::Operator::QuestionMark))
            .leaf()
            .chain(object_type.clone())
            .tree(SyntaxKind::UriParams);

        let uri_template = uri_path
            .chain(uri_params.or_not())
            .tree(SyntaxKind::UriTemplate);

        let uri_type = just_(TokenKind::Keyword(lex::Keyword::Primitive(
            lex::Primitive::Uri,
        )))
        .leaf()
        .or(uri_template);

        let array_type = just_(TokenKind::Control(lex::Control::ArrayBegin))
            .leaf()
            .chain(expr.clone())
            .chain(just_(TokenKind::Control(lex::Control::ArrayEnd)).leaf())
            .tree(SyntaxKind::Array);

        let prop_type = just_(TokenKind::Property)
            .leaf()
            .chain(expr.clone())
            .tree(SyntaxKind::Property);

        let content_prop = match_! { TokenKind::Keyword(lex::Keyword::Content(_)) }
            .leaf()
            .chain(just_(TokenKind::Operator(lex::Operator::Equal)).leaf())
            .chain(expr.clone());

        let content_type = just_(TokenKind::Control(lex::Control::ContentBegin))
            .leaf()
            .chain(
                content_prop
                    .clone()
                    .chain(just_(TokenKind::Control(lex::Control::Comma)).leaf())
                    .repeated()
                    .flatten(),
            )
            .chain(expr.clone().or_not())
            .chain(just_(TokenKind::Control(lex::Control::ContentEnd)).leaf())
            .tree(SyntaxKind::Content);

        let paren_type = just_(TokenKind::Control(lex::Control::ParenthesisBegin))
            .leaf()
            .chain(expr.clone())
            .chain(just_(TokenKind::Control(lex::Control::ParenthesisEnd)).leaf())
            .tree(SyntaxKind::SubExpression);

        let term_type = literal_type
            .or(prim_type)
            .or(uri_type)
            .or(array_type)
            .or(prop_type)
            .or(object_type.clone())
            .or(content_type)
            .or(paren_type)
            .or(variable)
            .chain(inline_ann.or_not())
            .tree(SyntaxKind::Terminal);

        let apply = just_(TokenKind::Identifier(lex::Identifier::Value))
            .leaf()
            .chain(term_type.clone().repeated().at_least(1))
            .tree(SyntaxKind::Application);

        let app_type = apply.or(term_type.clone());

        let range_type = variadic_op(lex::Operator::DoubleColon, app_type);

        let join_type = variadic_op(lex::Operator::Ampersand, range_type.clone());

        let any_type = variadic_op(lex::Operator::Tilde, join_type);

        let sum_type = variadic_op(lex::Operator::VerticalBar, any_type);

        let xfer = methods
            .chain(object_type.or_not())
            .chain(
                just_(TokenKind::Operator(lex::Operator::Colon))
                    .leaf()
                    .chain(term_type.clone())
                    .or_not()
                    .flatten(),
            )
            .chain(just_(TokenKind::Operator(lex::Operator::Arrow)).leaf())
            .chain(range_type)
            .tree(SyntaxKind::Transfer);

        let xfer_type = xfer.or(sum_type);

        let rel_type = term_type
            .chain(just_(TokenKind::Control(lex::Control::ParenthesisBegin)).leaf())
            .chain(xfer_type.clone())
            .chain(
                just_(TokenKind::Control(lex::Control::Comma))
                    .leaf()
                    .chain(xfer_type.clone())
                    .repeated()
                    .flatten(),
            )
            .chain(just_(TokenKind::Control(lex::Control::ParenthesisEnd)).leaf())
            .tree(SyntaxKind::Relation);

        rel_type.or(xfer_type)
    });

    let declaration = just_(TokenKind::Keyword(lex::Keyword::Let))
        .leaf()
        .chain(variable)
        .chain(binding.repeated())
        .chain(just_(TokenKind::Operator(lex::Operator::Equal)).leaf())
        .chain(expr_type.clone())
        .chain(just_(TokenKind::Control(lex::Control::Semicolon)).leaf())
        .tree(SyntaxKind::Declaration);

    let resource = just_(TokenKind::Keyword(lex::Keyword::Res))
        .leaf()
        .chain(expr_type)
        .chain(just_(TokenKind::Control(lex::Control::Semicolon)).leaf())
        .tree(SyntaxKind::Resource);

    let import = just_(TokenKind::Keyword(lex::Keyword::Use))
        .leaf()
        .chain(just_(TokenKind::Literal(lex::Literal::String)).leaf())
        .chain(just_(TokenKind::Control(lex::Control::Semicolon)).leaf())
        .tree(SyntaxKind::Import);

    let statement = line_ann
        .or(declaration)
        .or(resource)
        .or(import)
        .recover_with(skip_then_retry_until([]));

    statement.repeated().tree(SyntaxKind::Program)
}
