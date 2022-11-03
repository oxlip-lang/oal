use crate::atom::Ident;
use crate::rewrite::lexer::{
    Control, Identifier, Keyword, Lex, Literal, Operator, Path, Primitive, TokenKind, TokenValue,
};
use chumsky::prelude::*;
use oal_model::grammar::*;
use oal_model::lexicon::{Interner, TokenList};
use oal_model::syntax_nodes;
use std::fmt::Debug;

#[derive(Copy, Clone, Default, Debug)]
pub struct Gram;

impl Grammar for Gram {
    type Lex = Lex;
    type Kind = SyntaxKind;
}

impl Gram {
    pub fn parse<T>(tokens: TokenList<Lex>) -> SyntaxTree<T, Gram>
    where
        T: Clone + Default,
    {
        let (root, _) = parser::<T>().parse_recovery(
            tokens.stream(|k| !matches!(k, TokenKind::Space | TokenKind::Comment(_))),
        );

        let root = root.unwrap();

        SyntaxTree::import(tokens, root)
    }
}

#[derive(Debug)]
pub struct Term<'a, T>(NodeRef<'a, T, Gram>);

#[allow(dead_code)]
impl<'a, T> Term<'a, T>
where
    T: Default + Clone,
{
    pub fn cast(node: NodeRef<'a, T, Gram>) -> Option<Self> {
        match node.syntax().trunk() {
            SyntaxTrunk::Leaf(_) => Some(Term(node)),
            _ => None,
        }
    }

    pub fn node(&self) -> NodeRef<'a, T, Gram> {
        self.0
    }

    pub fn kind(&self) -> TokenKind {
        match self.node().syntax().trunk() {
            SyntaxTrunk::Leaf((kind, _)) => *kind,
            _ => unreachable!(),
        }
    }

    pub fn to_ident(&self) -> Ident {
        match self.node().token().value() {
            TokenValue::Symbol(sym) => self.node().tree().resolve(*sym).into(),
            _ => panic!("identifier must be a symbol"),
        }
    }
}

syntax_nodes!(
    Gram,
    Property,
    Array,
    Lambda,
    Symbol,
    Expression,
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

impl<'a, T> Symbol<'a, T>
where
    T: Default + Clone,
{
    pub fn term(&'a self) -> Term<'a, T> {
        Term::cast(self.node().first()).unwrap()
    }
}

impl<'a, T> Program<'a, T>
where
    T: Default + Clone,
{
    pub fn resources(&'a self) -> impl Iterator<Item = Resource<T>> {
        self.node().children().filter_map(Resource::cast)
    }

    pub fn declarations(&'a self) -> impl Iterator<Item = Declaration<T>> {
        self.node().children().filter_map(Declaration::cast)
    }

    pub fn imports(&'a self) -> impl Iterator<Item = Import<T>> {
        self.node().children().filter_map(Import::cast)
    }
}

impl<'a, T> Declaration<'a, T>
where
    T: Default + Clone,
{
    // TODO: get the real values for this
    const NAME_POS: usize = 1;
    const RHS_POS: usize = 2;

    pub fn name(&'a self) -> Ident {
        let name = self.node().nth(Self::NAME_POS);
        if let Some(term) = Term::cast(name) {
            term.to_ident()
        } else {
            panic!("declaration name must be a terminal")
        }
    }

    pub fn rhs(&'a self) -> Expression<T> {
        let rhs = self.node().nth(Self::RHS_POS);
        if let Some(expr) = Expression::cast(rhs) {
            expr
        } else {
            panic!("declaration rhs must be an expression")
        }
    }
}

impl<'a, T> Import<'a, T>
where
    T: Default + Clone,
{
    const MODULE_POS: usize = 1;

    pub fn module(&'a self) -> &'a str {
        match self.node().nth(Self::MODULE_POS).token().value() {
            TokenValue::Symbol(sym) => self.node().tree().resolve(*sym),
            _ => panic!("module must be a symbol"),
        }
    }
}

fn just_<E>(kind: TokenKind) -> impl Parser<SyntaxToken<Lex>, SyntaxToken<Lex>, Error = E> + Clone
where
    E: chumsky::Error<SyntaxToken<Lex>>,
{
    just_token::<_, Gram>(kind)
}

fn variadic_op<'a, P, E, T>(
    op: Operator,
    p: P,
) -> impl Parser<SyntaxToken<Lex>, ParseNode<T, Gram>, Error = E> + Clone + 'a
where
    P: Parser<SyntaxToken<Lex>, ParseNode<T, Gram>, Error = E> + Clone + 'a,
    E: chumsky::Error<SyntaxToken<Lex>> + 'a,
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

fn parser<'a, T>(
) -> impl Parser<SyntaxToken<Lex>, ParseNode<T, Gram>, Error = Simple<SyntaxToken<Lex>>> + 'a
where
    T: Default + Clone + 'a,
{
    let annotation = select! { t@(TokenKind::Annotation(_), _) => t }.leaf();

    let import = just_(TokenKind::Keyword(Keyword::Use))
        .leaf()
        .chain(just_(TokenKind::Literal(Literal::String)).leaf())
        .tree(SyntaxKind::Import);

    let expr_type = recursive(|expr| {
        let literal_type = select! { t@(TokenKind::Literal(_), _) => t }.leaf();

        let object_type = just_(TokenKind::Control(Control::BlockBegin))
            .leaf()
            .chain(
                expr.clone()
                    .chain(
                        just_(TokenKind::Control(Control::Comma))
                            .leaf()
                            .chain(expr.clone())
                            .repeated()
                            .flatten(),
                    )
                    .or_not()
                    .flatten(),
            )
            .chain(just_(TokenKind::Control(Control::BlockEnd)).leaf())
            .tree(SyntaxKind::Object);

        let uri_var = just_(TokenKind::Control(Control::BlockBegin))
            .leaf()
            .chain(expr.clone())
            .chain(just_(TokenKind::Control(Control::BlockEnd)).leaf())
            .tree(SyntaxKind::UriVariable);

        let uri_root = just_(TokenKind::Literal(Literal::Path(Path::Root))).leaf();
        let uri_segment = just_(TokenKind::Literal(Literal::Path(Path::Segment))).leaf();

        let uri_path = uri_segment
            .map(|l| vec![l])
            .or(uri_root.chain(uri_var.or_not()))
            .repeated()
            .at_least(1)
            .flatten()
            .collect()
            .tree(SyntaxKind::UriPath);

        let uri_params = just_(TokenKind::Operator(Operator::QuestionMark))
            .leaf()
            .chain(object_type.clone())
            .tree(SyntaxKind::UriParams);

        let uri_template = uri_path
            .chain(uri_params.or_not())
            .tree(SyntaxKind::UriTemplate);

        let uri_type = just_(TokenKind::Keyword(Keyword::Primitive(Primitive::Uri)))
            .leaf()
            .or(uri_template);

        let prim_type = select! { t@(TokenKind::Keyword(Keyword::Primitive(_)), _) => t }.leaf();

        let array_type = just_(TokenKind::Control(Control::ArrayBegin))
            .leaf()
            .chain(expr.clone())
            .chain(just_(TokenKind::Control(Control::ArrayEnd)).leaf())
            .tree(SyntaxKind::Array);

        let prop_type = just_(TokenKind::Literal(Literal::Property))
            .leaf()
            .chain(expr)
            .tree(SyntaxKind::Property);

        let term_type = prim_type
            .or(uri_type)
            .or(array_type)
            .or(prop_type)
            .or(object_type.clone())
            .or(literal_type);

        let apply = just_(TokenKind::Identifier(Identifier::Value))
            .leaf()
            .chain(term_type.clone().repeated().at_least(1))
            .tree(SyntaxKind::Application);

        let app_type = apply.or(term_type.clone());

        let range_type = variadic_op(Operator::DoubleColon, app_type);

        let join_type = variadic_op(Operator::Ampersand, range_type.clone());

        let any_type = variadic_op(Operator::Tilde, join_type);

        let sum_type = variadic_op(Operator::VerticalBar, any_type);

        let method = select! { t@(TokenKind::Keyword(Keyword::Method(_)), _) => t }.leaf();

        let methods = method.chain(
            just_(TokenKind::Control(Control::Comma))
                .leaf()
                .chain(method)
                .repeated()
                .flatten(),
        );

        let xfer = methods
            .chain(object_type.or_not())
            .chain(
                just_(TokenKind::Operator(Operator::Colon))
                    .leaf()
                    .chain(term_type.clone())
                    .or_not()
                    .flatten(),
            )
            .chain(just_(TokenKind::Operator(Operator::Arrow)).leaf())
            .chain(range_type)
            .tree(SyntaxKind::Transfer);

        let xfer_type = xfer.or(sum_type);

        let rel_type = term_type
            .chain(just_(TokenKind::Control(Control::ParenthesisBegin)).leaf())
            .chain(xfer_type.clone())
            .chain(
                just_(TokenKind::Control(Control::Comma))
                    .leaf()
                    .chain(xfer_type.clone())
                    .repeated()
                    .flatten(),
            )
            .chain(just_(TokenKind::Control(Control::ParenthesisEnd)).leaf())
            .tree(SyntaxKind::Relation);

        rel_type.or(xfer_type)
    });

    let resource = just_(TokenKind::Keyword(Keyword::Res))
        .leaf()
        .chain(expr_type)
        .chain(just_(TokenKind::Control(Control::Semicolon)).leaf())
        .tree(SyntaxKind::Resource);

    let statement = annotation
        .or(import)
        .or(resource)
        .recover_with(skip_then_retry_until([]));

    statement.repeated().tree(SyntaxKind::Program)
}
