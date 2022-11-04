use crate::atom;
use crate::errors::Result;
use chumsky::prelude::*;
use oal_model::lexicon::*;
use std::fmt::Debug;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Put,
    Post,
    Patch,
    Delete,
    Options,
    Head,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Primitive {
    Num,
    Str,
    Uri,
    Bool,
    Int,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Content {
    Media,
    Headers,
    Status,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Keyword {
    Method(Method),
    Primitive(Primitive),
    Content(Content),
    Let,
    Res,
    Use,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Control {
    BlockBegin,
    BlockEnd,
    ParenthesisBegin,
    ParenthesisEnd,
    ArrayBegin,
    ArrayEnd,
    ContentBegin,
    ContentEnd,
    Semicolon,
    Comma,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operator {
    Equal,
    Colon,
    DoubleColon,
    QuestionMark,
    Arrow,
    Ampersand,
    Tilde,
    VerticalBar,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Path {
    Root,
    Segment,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Literal {
    HttpStatus,
    Number,
    String,
    Path(Path),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Comment {
    Line,
    Block,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Annotation {
    Line,
    Inline,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Identifier {
    Value,
    Reference,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Space,
    Property,
    Comment(Comment),
    Annotation(Annotation),
    Identifier(Identifier),
    Keyword(Keyword),
    Literal(Literal),
    Control(Control),
    Operator(Operator),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TokenValue {
    None,
    HttpStatus(atom::HttpStatus),
    Number(u64),
    String(String),
    Symbol(Symbol),
}

impl Interned for TokenValue {
    fn copy<I: Interner>(&self, from: &I, to: &mut I) -> Self {
        match self {
            TokenValue::Symbol(sym) => TokenValue::Symbol(to.register(from.resolve(*sym))),
            TokenValue::String(s) => TokenValue::String(s.clone()),
            TokenValue::Number(n) => TokenValue::Number(*n),
            TokenValue::HttpStatus(s) => TokenValue::HttpStatus(*s),
            TokenValue::None => TokenValue::None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct Lex;

impl Lexicon for Lex {
    type Kind = TokenKind;
    type Value = TokenValue;
}

impl Lex {
    pub fn tokenize(input: &str) -> Result<TokenList<Self>> {
        let mut token_list = TokenList::default();

        let (tokens, mut errs) = lexer().parse_recovery(input);

        if !errs.is_empty() {
            Err(errs.swap_remove(0).into())
        } else {
            if let Some(tokens) = tokens {
                // Note: Chumsky does not support stateful combinators at the moment.
                // Therefore we need a second pass over the vector of tokens to
                // internalize the strings and build the index list.
                tokens.into_iter().for_each(|((kind, value), span)| {
                    let value = match value {
                        TokenValue::String(s) => TokenValue::Symbol(token_list.register(s)),
                        _ => value,
                    };
                    token_list.push(((kind, value), span));
                });
            }
            Ok(token_list)
        }
    }
}

fn lexer() -> impl Parser<char, Vec<TokenSpan<Lex>>, Error = Simple<char>> {
    let ident_chars =
        filter(|c: &char| c.is_ascii_alphanumeric() || *c == '$' || *c == '-' || *c == '_')
            .repeated();

    let val_ident = filter(|c: &char| c.is_ascii_alphabetic())
        .chain(ident_chars)
        .collect::<String>()
        .map(|i| {
            (
                TokenKind::Identifier(Identifier::Value),
                TokenValue::String(i),
            )
        });

    let ref_ident = just('@')
        .chain(ident_chars.at_least(1))
        .collect::<String>()
        .map(|i| {
            (
                TokenKind::Identifier(Identifier::Reference),
                TokenValue::String(i),
            )
        });

    let ident = val_ident.or(ref_ident);

    let keyword = text::ident()
        .try_map(|k: String, span| match k.as_str() {
            "let" => Ok(Keyword::Let),
            "res" => Ok(Keyword::Res),
            "use" => Ok(Keyword::Use),
            "num" => Ok(Keyword::Primitive(Primitive::Num)),
            "str" => Ok(Keyword::Primitive(Primitive::Str)),
            "uri" => Ok(Keyword::Primitive(Primitive::Uri)),
            "bool" => Ok(Keyword::Primitive(Primitive::Bool)),
            "int" => Ok(Keyword::Primitive(Primitive::Int)),
            "get" => Ok(Keyword::Method(Method::Get)),
            "put" => Ok(Keyword::Method(Method::Put)),
            "post" => Ok(Keyword::Method(Method::Post)),
            "patch" => Ok(Keyword::Method(Method::Patch)),
            "delete" => Ok(Keyword::Method(Method::Delete)),
            "options" => Ok(Keyword::Method(Method::Options)),
            "head" => Ok(Keyword::Method(Method::Head)),
            "media" => Ok(Keyword::Content(Content::Media)),
            "headers" => Ok(Keyword::Content(Content::Headers)),
            "status" => Ok(Keyword::Content(Content::Status)),
            _ => Err(Simple::custom(span, "not a keyword")),
        })
        .map(|k| (TokenKind::Keyword(k), TokenValue::None));

    let property = just('\'')
        .ignore_then(
            filter(|c: &char| {
                c.is_ascii_alphanumeric() || *c == '$' || *c == '-' || *c == '_' || *c == '@'
            })
            .repeated()
            .at_least(1),
        )
        .collect::<String>()
        .map(|p| (TokenKind::Property, TokenValue::String(p)));

    let http_status_range = one_of("12345")
        .then_ignore(just("XX"))
        .map(|c| match c {
            '1' => atom::HttpStatusRange::Info,
            '2' => atom::HttpStatusRange::Success,
            '3' => atom::HttpStatusRange::Redirect,
            '4' => atom::HttpStatusRange::ClientError,
            '5' => atom::HttpStatusRange::ServerError,
            _ => unreachable!(),
        })
        .map(|r| {
            (
                TokenKind::Literal(Literal::HttpStatus),
                TokenValue::HttpStatus(atom::HttpStatus::Range(r)),
            )
        });

    let lit_num = text::int(10).map(|n: String| {
        (
            TokenKind::Literal(Literal::Number),
            TokenValue::Number(n.parse().unwrap()),
        )
    });

    let lit_str = just('"')
        .ignore_then(filter(|c| *c != '"').repeated())
        .then_ignore(just('"'))
        .collect()
        .map(|s| (TokenKind::Literal(Literal::String), TokenValue::String(s)));

    let lit_path = just('/')
        .ignore_then(
            filter(|c: &char| {
                c.is_ascii_alphanumeric()
                    || *c == '-'
                    || *c == '.'
                    || *c == '_'
                    || *c == '~'
                    || *c == '%'
            })
            .repeated(),
        )
        .collect::<String>()
        .map(|p| {
            if p.is_empty() {
                (
                    TokenKind::Literal(Literal::Path(Path::Root)),
                    TokenValue::None,
                )
            } else {
                (
                    TokenKind::Literal(Literal::Path(Path::Segment)),
                    TokenValue::String(p),
                )
            }
        });

    let literal = lit_str
        .or(property)
        .or(http_status_range)
        .or(lit_num)
        .or(lit_path);

    let operator = just("->")
        .to(Operator::Arrow)
        .or(just("::").to(Operator::DoubleColon))
        .or(just('?').to(Operator::QuestionMark))
        .or(just(':').to(Operator::Colon))
        .or(just('=').to(Operator::Equal))
        .or(just('&').to(Operator::Ampersand))
        .or(just('~').to(Operator::Tilde))
        .or(just('|').to(Operator::VerticalBar))
        .map(|p| (TokenKind::Operator(p), TokenValue::None));

    let control = select! {
        '{' => Control::BlockBegin,
        '}' => Control::BlockEnd,
        '(' => Control::ParenthesisBegin,
        ')' => Control::ParenthesisEnd,
        '[' => Control::ArrayBegin,
        ']' => Control::ArrayEnd,
        '<' => Control::ContentBegin,
        '>' => Control::ContentEnd,
        ';' => Control::Semicolon,
        ',' => Control::Comma,
    }
    .map(|c| (TokenKind::Control(c), TokenValue::None));

    let space = filter(|c: &char| c.is_whitespace())
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(|s| (TokenKind::Space, TokenValue::String(s)));

    let newline = one_of("\r\n").repeated().at_least(1);

    let line_comment = just("//")
        .ignore_then(take_until(newline.clone()))
        .map(|(c, n)| {
            (
                TokenKind::Comment(Comment::Line),
                TokenValue::String(c.into_iter().chain(n.into_iter()).collect()),
            )
        });

    let block_comment = just("/*")
        .ignore_then(take_until(just("*/")))
        .map(|(c, _)| {
            (
                TokenKind::Comment(Comment::Block),
                TokenValue::String(c.into_iter().collect()),
            )
        });

    let comment = line_comment.or(block_comment);

    let line_ann = just('#').ignore_then(take_until(newline)).map(|(c, n)| {
        (
            TokenKind::Annotation(Annotation::Line),
            TokenValue::String(c.into_iter().chain(n.into_iter()).collect()),
        )
    });

    let inline_ann = just('`')
        .ignore_then(filter(|c| *c != '`').repeated())
        .then_ignore(just('`'))
        .collect()
        .map(|a| {
            (
                TokenKind::Annotation(Annotation::Inline),
                TokenValue::String(a),
            )
        });

    let annotation = line_ann.or(inline_ann);

    let token = space
        .or(comment)
        .or(annotation)
        .or(control)
        .or(operator)
        .or(literal)
        .or(keyword)
        .or(ident)
        .recover_with(skip_then_retry_until([]));

    token
        .map_with_span(|t, s| (t, s))
        .repeated()
        .then_ignore(end())
}
