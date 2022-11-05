use crate::atom;
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Token(TokenKind, TokenValue);

impl Lexeme for Token {
    type Kind = TokenKind;
    type Value = TokenValue;

    fn new(kind: TokenKind, value: TokenValue) -> Self {
        Token(kind, value)
    }

    fn kind(&self) -> TokenKind {
        self.0
    }

    fn value(&self) -> &TokenValue {
        &self.1
    }

    fn is_trivia(&self) -> bool {
        matches!(self.0, TokenKind::Space | TokenKind::Comment(_))
    }

    fn internalize<I: Interner>(self, i: &mut I) -> Self {
        let value = match self.1 {
            TokenValue::String(s) => TokenValue::Symbol(i.register(s)),
            v => v,
        };
        Self::new(self.0, value)
    }
}

pub fn lexer() -> impl Parser<char, Vec<TokenSpan<Token>>, Error = Simple<char>> {
    let ident_chars =
        filter(|c: &char| c.is_ascii_alphanumeric() || *c == '$' || *c == '-' || *c == '_')
            .repeated();

    let val_ident = filter(|c: &char| c.is_ascii_alphabetic())
        .chain(ident_chars)
        .collect::<String>()
        .map(|i| {
            Token::new(
                TokenKind::Identifier(Identifier::Value),
                TokenValue::String(i),
            )
        });

    let ref_ident = just('@')
        .chain(ident_chars.at_least(1))
        .collect::<String>()
        .map(|i| {
            Token::new(
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
        .map(|k| Token::new(TokenKind::Keyword(k), TokenValue::None));

    let property = just('\'')
        .ignore_then(
            filter(|c: &char| {
                c.is_ascii_alphanumeric() || *c == '$' || *c == '-' || *c == '_' || *c == '@'
            })
            .repeated()
            .at_least(1),
        )
        .collect::<String>()
        .map(|p| Token::new(TokenKind::Property, TokenValue::String(p)));

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
            Token::new(
                TokenKind::Literal(Literal::HttpStatus),
                TokenValue::HttpStatus(atom::HttpStatus::Range(r)),
            )
        });

    let lit_num = text::int(10).map(|n: String| {
        Token::new(
            TokenKind::Literal(Literal::Number),
            TokenValue::Number(n.parse().unwrap()),
        )
    });

    let lit_str = just('"')
        .ignore_then(filter(|c| *c != '"').repeated())
        .then_ignore(just('"'))
        .collect()
        .map(|s| Token::new(TokenKind::Literal(Literal::String), TokenValue::String(s)));

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
                Token::new(
                    TokenKind::Literal(Literal::Path(Path::Root)),
                    TokenValue::None,
                )
            } else {
                Token::new(
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
        .map(|p| Token::new(TokenKind::Operator(p), TokenValue::None));

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
    .map(|c| Token::new(TokenKind::Control(c), TokenValue::None));

    let space = filter(|c: &char| c.is_whitespace())
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(|s| Token::new(TokenKind::Space, TokenValue::String(s)));

    let newline = one_of("\r\n").repeated().at_least(1);

    let line_comment = just("//")
        .ignore_then(take_until(newline.clone()))
        .map(|(c, n)| {
            Token::new(
                TokenKind::Comment(Comment::Line),
                TokenValue::String(c.into_iter().chain(n.into_iter()).collect()),
            )
        });

    let block_comment = just("/*")
        .ignore_then(take_until(just("*/")))
        .map(|(c, _)| {
            Token::new(
                TokenKind::Comment(Comment::Block),
                TokenValue::String(c.into_iter().collect()),
            )
        });

    let comment = line_comment.or(block_comment);

    let line_ann = just('#').ignore_then(take_until(newline)).map(|(c, n)| {
        Token::new(
            TokenKind::Annotation(Annotation::Line),
            TokenValue::String(c.into_iter().chain(n.into_iter()).collect()),
        )
    });

    let inline_ann = just('`')
        .ignore_then(filter(|c| *c != '`').repeated())
        .then_ignore(just('`'))
        .collect()
        .map(|a| {
            Token::new(
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
