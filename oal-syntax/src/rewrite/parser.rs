use crate::atom::Ident;
use crate::rewrite::lexer as lex;
use crate::rewrite::lexer::{Token, TokenKind, TokenValue};
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

terminal_node!(Gram, Symbol, TokenKind::Identifier(_));

impl<'a, T: Default + Clone> Symbol<'a, T> {
    pub fn as_ident(&self) -> Ident {
        self.node().as_str().into()
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
    pub fn as_str(&self) -> &'a str {
        match self.node().token().kind() {
            TokenKind::PathElement(lex::PathElement::Root) => "/",
            TokenKind::PathElement(lex::PathElement::Segment) => self.node().as_str(),
            _ => unreachable!(),
        }
    }
}

terminal_node!(Gram, ProperyName, TokenKind::Property);

impl<'a, T: Default + Clone> ProperyName<'a, T> {
    pub fn as_ident(&self) -> Ident {
        self.node().as_str().into()
    }
}

terminal_node!(Gram, Method, TokenKind::Keyword(lex::Keyword::Method(_)));

impl<'a, T: Default + Clone> Method<'a, T> {
    pub fn method(&self) -> lex::Method {
        let TokenKind::Keyword(lex::Keyword::Method(m)) = self.node().token().kind() else { unreachable!() };
        m
    }
}

terminal_node!(Gram, Literal, TokenKind::Literal(_));

impl<'a, T: Default + Clone> Literal<'a, T> {
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

impl<'a, T: Default + Clone> ContentTag<'a, T> {
    pub fn tag(&self) -> lex::Content {
        let TokenKind::Keyword(lex::Keyword::Content(t)) = self.node().token().kind() else { unreachable!() };
        t
    }
}

syntax_nodes!(
    Gram,
    Terminal,
    SubExpression,
    ContentMeta,
    ContentMetaList,
    ContentBody,
    Content,
    Property,
    Array,
    Annotations,
    Declaration,
    UriVariable,
    UriPath,
    UriParams,
    UriTemplate,
    Object,
    Application,
    VariadicOp,
    XferMethods,
    XferParams,
    XferDomain,
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

impl<'a, T: Default + Clone> Annotations<'a, T> {
    pub fn items(&self) -> impl Iterator<Item = &'a str> {
        self.node().children().map(|c| c.as_str())
    }
}

impl<'a, T: Default + Clone> Declaration<'a, T> {
    const ANN_POS: usize = 0;
    const SYM_POS: usize = 2;
    const RHS_POS: usize = 4;

    pub fn annotations(&self) -> impl Iterator<Item = &'a str> {
        Annotations::cast(self.node().nth(Self::ANN_POS))
            .expect("expected annotations")
            .items()
    }

    pub fn symbol(&self) -> Symbol<'a, T> {
        Symbol::cast(self.node().nth(Self::SYM_POS)).expect("declaration lhs must be a symbol")
    }

    pub fn rhs(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::RHS_POS)
    }
}

impl<'a, T: Default + Clone> Import<'a, T> {
    const MODULE_POS: usize = 1;

    pub fn module(&self) -> &'a str {
        self.node().nth(Self::MODULE_POS).as_str()
    }
}

impl<'a, T: Default + Clone> Terminal<'a, T> {
    const INNER_POS: usize = 0;
    const ANN_POS: usize = 1;

    pub fn inner(&self) -> NodeRef<'a, T, Gram> {
        self.node().nth(Self::INNER_POS)
    }

    pub fn annotation(&self) -> Option<&'a str> {
        self.node()
            .children()
            .nth(Self::ANN_POS)
            .map(|n| n.as_str())
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
            .map(|inner| UriParams::cast(inner).expect("expected URI parameters"))
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
        Object::cast(self.node().nth(Self::INNER_POS))
            .expect("URI parameters must be an object")
            .properties()
    }
}

impl<'a, T: Default + Clone> Property<'a, T> {
    const NAME_POS: usize = 0;
    const RHS_POS: usize = 1;

    pub fn name(&self) -> ProperyName<'a, T> {
        ProperyName::cast(self.node().nth(Self::NAME_POS)).expect("expected a propery name")
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

impl<'a, T: Default + Clone> XferMethods<'a, T> {
    pub fn methods(&self) -> impl Iterator<Item = Method<'a, T>> {
        self.node().children().filter_map(Method::cast)
    }
}

impl<'a, T: Default + Clone> XferParams<'a, T> {
    const INNER_POS: usize = 0;

    pub fn params(&self) -> Option<impl Iterator<Item = NodeRef<'a, T, Gram>>> {
        self.node().children().nth(Self::INNER_POS).map(|inner| {
            Object::cast(inner)
                .expect("transfer parameters must be an object")
                .properties()
        })
    }
}

impl<'a, T: Default + Clone> XferDomain<'a, T> {
    const INNER_POS: usize = 1;

    pub fn inner(&self) -> Option<Terminal<'a, T>> {
        self.node()
            .children()
            .nth(Self::INNER_POS)
            .map(|inner| Terminal::cast(inner).expect("transfer domain must be a terminal"))
    }
}

impl<'a, T: Default + Clone> Transfer<'a, T> {
    const METHODS_POS: usize = 0;
    const PARAMS_POS: usize = 1;
    const DOMAIN_POS: usize = 2;
    const RANGE_POS: usize = 4;

    pub fn methods(&self) -> impl Iterator<Item = Method<'a, T>> {
        XferMethods::cast(self.node().nth(Self::METHODS_POS))
            .expect("expected transfer methods")
            .methods()
    }

    pub fn params(&self) -> Option<impl Iterator<Item = NodeRef<'a, T, Gram>>> {
        XferParams::cast(self.node().nth(Self::PARAMS_POS))
            .expect("expected transfer parameters")
            .params()
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

impl<'a, T: Default + Clone> VariadicOp<'a, T> {
    const OPERATOR_POS: usize = 1;

    pub fn operator(&self) -> lex::Operator {
        let TokenKind::Operator(op) = self.node().nth(Self::OPERATOR_POS).token().kind() else { panic!("expected an operator") };
        op
    }

    pub fn operands(&self) -> impl Iterator<Item = NodeRef<'a, T, Gram>> {
        self.node().children().step_by(2)
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

impl<'a, T: Default + Clone> ContentMeta<'a, T> {
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

impl<'a, T: Default + Clone> ContentMetaList<'a, T> {
    pub fn items(&self) -> impl Iterator<Item = ContentMeta<'a, T>> {
        self.node().children().filter_map(ContentMeta::cast)
    }
}

impl<'a, T: Default + Clone> ContentBody<'a, T> {
    const INNER_POS: usize = 0;

    pub fn inner(&self) -> Option<NodeRef<'a, T, Gram>> {
        self.node().children().nth(Self::INNER_POS)
    }
}

impl<'a, T: Default + Clone> Content<'a, T> {
    const META_POS: usize = 1;
    const BODY_POS: usize = 2;

    pub fn meta(&self) -> impl Iterator<Item = ContentMeta<'a, T>> {
        ContentMetaList::cast(self.node().nth(Self::META_POS))
            .expect("expected content meta")
            .items()
    }

    pub fn body(&self) -> Option<NodeRef<'a, T, Gram>> {
        ContentBody::cast(self.node().nth(Self::BODY_POS))
            .expect("expected content body")
            .inner()
    }
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

pub fn parser<'a, T>(
) -> impl Parser<TokenAlias<Token>, ParseNode<T, Gram>, Error = Simple<TokenAlias<Token>>> + 'a
where
    T: Default + Clone + 'a,
{
    let binding = just_token(TokenKind::Identifier(lex::Identifier::Value));

    let variable = match_token! { TokenKind::Identifier(_) };

    let literal_type = match_token! { TokenKind::Literal(_) };

    let prim_type = match_token! { TokenKind::Keyword(lex::Keyword::Primitive(_)) };

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

    let line_ann = just_token(TokenKind::Annotation(lex::Annotation::Line));

    let inline_ann = just_token(TokenKind::Annotation(lex::Annotation::Inline));

    let expr_type = recursive(|expr| {
        let object_type = tree_many(
            just_token(TokenKind::Control(lex::Control::BlockBegin))
                .chain(
                    expr.clone()
                        .chain(
                            just_token(TokenKind::Control(lex::Control::Comma))
                                .chain(expr.clone())
                                .repeated()
                                .flatten(),
                        )
                        .or_not()
                        .flatten(),
                )
                .chain(just_token(TokenKind::Control(lex::Control::BlockEnd))),
            SyntaxKind::Object,
        );

        let uri_var = tree_many(
            just_token(TokenKind::Control(lex::Control::BlockBegin))
                .chain(expr.clone())
                .chain(just_token(TokenKind::Control(lex::Control::BlockEnd))),
            SyntaxKind::UriVariable,
        );

        let uri_path = tree_many(
            uri_segment
                .map(|l| vec![l])
                .or(uri_root.chain(uri_var.or_not()))
                .repeated()
                .at_least(1)
                .flatten()
                .collect(),
            SyntaxKind::UriPath,
        );

        let uri_params = tree_many(
            just_token(TokenKind::Operator(lex::Operator::QuestionMark)).chain(object_type.clone()),
            SyntaxKind::UriParams,
        );

        let uri_template = tree_many(uri_path.chain(uri_params.or_not()), SyntaxKind::UriTemplate);

        let uri_type = just_token(TokenKind::Keyword(lex::Keyword::Primitive(
            lex::Primitive::Uri,
        )))
        .or(uri_template);

        let array_type = tree_many(
            just_token(TokenKind::Control(lex::Control::ArrayBegin))
                .chain(expr.clone())
                .chain(just_token(TokenKind::Control(lex::Control::ArrayEnd))),
            SyntaxKind::Array,
        );

        let property_type = tree_many(
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
            content_meta
                .chain(just_token(TokenKind::Control(lex::Control::Comma)))
                .repeated()
                .flatten(),
            SyntaxKind::ContentMetaList,
        );

        let content_body = tree_maybe(expr.clone().or_not(), SyntaxKind::ContentBody);

        let content_type = tree_many(
            just_token(TokenKind::Control(lex::Control::ContentBegin))
                .chain(content_meta_list)
                .chain(content_body)
                .chain(just_token(TokenKind::Control(lex::Control::ContentEnd))),
            SyntaxKind::Content,
        );

        let subexpr_type = tree_many(
            just_token(TokenKind::Control(lex::Control::ParenthesisBegin))
                .chain(expr.clone())
                .chain(just_token(TokenKind::Control(lex::Control::ParenthesisEnd))),
            SyntaxKind::SubExpression,
        );

        let term_type = tree_many(
            literal_type
                .or(prim_type)
                .or(uri_type)
                .or(array_type)
                .or(property_type)
                .or(object_type.clone())
                .or(content_type)
                .or(subexpr_type)
                .or(variable)
                .chain(inline_ann.or_not()),
            SyntaxKind::Terminal,
        );

        let apply = tree_many(
            just_token(TokenKind::Identifier(lex::Identifier::Value))
                .chain(term_type.clone().repeated().at_least(1)),
            SyntaxKind::Application,
        );

        let app_type = apply.or(term_type.clone());

        let range_type = variadic_op(lex::Operator::DoubleColon, app_type);

        let join_type = variadic_op(lex::Operator::Ampersand, range_type.clone());

        let any_type = variadic_op(lex::Operator::Tilde, join_type);

        let sum_type = variadic_op(lex::Operator::VerticalBar, any_type);

        let xfer_params = tree_maybe(object_type.or_not(), SyntaxKind::XferParams);

        let xfer_domain = tree_many(
            just_token(TokenKind::Operator(lex::Operator::Colon))
                .chain(term_type.clone())
                .or_not()
                .flatten(),
            SyntaxKind::XferDomain,
        );

        let transfer = tree_many(
            xfer_methods
                .chain(xfer_params)
                .chain(xfer_domain)
                .chain(just_token(TokenKind::Operator(lex::Operator::Arrow)))
                .chain(range_type),
            SyntaxKind::Transfer,
        );

        let xfer_type = transfer.or(sum_type);

        let rel_type = tree_many(
            term_type
                .chain(just_token(TokenKind::Control(
                    lex::Control::ParenthesisBegin,
                )))
                .chain(xfer_type.clone())
                .chain(
                    just_token(TokenKind::Control(lex::Control::Comma))
                        .chain(xfer_type.clone())
                        .repeated()
                        .flatten(),
                )
                .chain(just_token(TokenKind::Control(lex::Control::ParenthesisEnd))),
            SyntaxKind::Relation,
        );

        rel_type.or(xfer_type)
    });

    let annotations = tree_many(line_ann.repeated(), SyntaxKind::Annotations);

    let declaration = tree_many(
        annotations
            .chain(just_token(TokenKind::Keyword(lex::Keyword::Let)))
            .chain(variable)
            .chain(binding.repeated())
            .chain(just_token(TokenKind::Operator(lex::Operator::Equal)))
            .chain(expr_type.clone())
            .chain(just_token(TokenKind::Control(lex::Control::Semicolon))),
        SyntaxKind::Declaration,
    );

    let resource = tree_many(
        just_token(TokenKind::Keyword(lex::Keyword::Res))
            .chain(expr_type)
            .chain(just_token(TokenKind::Control(lex::Control::Semicolon))),
        SyntaxKind::Resource,
    );

    let import = tree_many(
        just_token(TokenKind::Keyword(lex::Keyword::Use))
            .chain(just_token(TokenKind::Literal(lex::Literal::String)))
            .chain(just_token(TokenKind::Control(lex::Control::Semicolon))),
        SyntaxKind::Import,
    );

    let statement = declaration
        .or(resource)
        .or(import)
        .recover_with(skip_then_retry_until([]));

    tree_many(statement.repeated(), SyntaxKind::Program)
}
