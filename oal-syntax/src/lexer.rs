use crate::atom;
use chumsky::prelude::*;
use oal_model::lexicon::*;
use std::fmt::Debug;

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
    Method(atom::Method),
    Primitive(Primitive),
    Content(Content),
    Let,
    Res,
    Use,
    Rec,
    On,
    As,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Control {
    BraceLeft,
    BraceRight,
    ParenLeft,
    ParenRight,
    BracketLeft,
    BracketRight,
    ChevronLeft,
    ChevronRight,
    Semicolon,
    Comma,
    FullStop,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operator {
    Equal,
    Colon,
    DoubleColon,
    QuestionMark,
    ExclamationMark,
    Arrow,
    Ampersand,
    Tilde,
    VerticalBar,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PathElement {
    Root,
    Segment,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Literal {
    HttpStatus,
    Number,
    String,
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
    PathElement(PathElement),
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

impl Intern for TokenValue {
    fn copy<I: Interner>(&self, from: &I, to: &mut I) -> Self {
        match self {
            TokenValue::Symbol(sym) => TokenValue::Symbol(to.register(from.resolve(*sym))),
            TokenValue::String(s) => TokenValue::String(s.clone()),
            TokenValue::Number(n) => TokenValue::Number(*n),
            TokenValue::HttpStatus(s) => TokenValue::HttpStatus(*s),
            TokenValue::None => TokenValue::None,
        }
    }

    fn as_str<'a, I: Interner>(&'a self, from: &'a I) -> &'a str {
        match self {
            TokenValue::String(str) => str.as_str(),
            TokenValue::Symbol(sym) => from.resolve(*sym),
            _ => panic!("not a string"),
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

pub fn lexer() -> impl Parser<char, Vec<TokenSpan<Token>>, Error = ParserError> {
    let ident_chars =
        filter(|c: &char| c.is_ascii_alphanumeric() || *c == '$' || *c == '-' || *c == '_')
            .repeated();

    let val_ident = filter(|c: &char| c.is_ascii_alphabetic() || *c == '_')
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
            "rec" => Ok(Keyword::Rec),
            "on" => Ok(Keyword::On),
            "as" => Ok(Keyword::As),
            "num" => Ok(Keyword::Primitive(Primitive::Num)),
            "str" => Ok(Keyword::Primitive(Primitive::Str)),
            "uri" => Ok(Keyword::Primitive(Primitive::Uri)),
            "bool" => Ok(Keyword::Primitive(Primitive::Bool)),
            "int" => Ok(Keyword::Primitive(Primitive::Int)),
            "get" => Ok(Keyword::Method(atom::Method::Get)),
            "put" => Ok(Keyword::Method(atom::Method::Put)),
            "post" => Ok(Keyword::Method(atom::Method::Post)),
            "patch" => Ok(Keyword::Method(atom::Method::Patch)),
            "delete" => Ok(Keyword::Method(atom::Method::Delete)),
            "options" => Ok(Keyword::Method(atom::Method::Options)),
            "head" => Ok(Keyword::Method(atom::Method::Head)),
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

    let literal_number = text::int(10).map(|n: String| {
        Token::new(
            TokenKind::Literal(Literal::Number),
            TokenValue::Number(n.parse().unwrap()),
        )
    });

    let literal_string = just('"')
        .ignore_then(filter(|c| *c != '"').repeated())
        .then_ignore(just('"'))
        .collect()
        .map(|s| Token::new(TokenKind::Literal(Literal::String), TokenValue::String(s)));

    let path_element = just('/')
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
                Token::new(TokenKind::PathElement(PathElement::Root), TokenValue::None)
            } else {
                Token::new(
                    TokenKind::PathElement(PathElement::Segment),
                    TokenValue::String(p),
                )
            }
        });

    let literal = literal_string.or(http_status_range).or(literal_number);

    let operator = just("->")
        .to(Operator::Arrow)
        .or(just("::").to(Operator::DoubleColon))
        .or(just('?').to(Operator::QuestionMark))
        .or(just('!').to(Operator::ExclamationMark))
        .or(just(':').to(Operator::Colon))
        .or(just('=').to(Operator::Equal))
        .or(just('&').to(Operator::Ampersand))
        .or(just('~').to(Operator::Tilde))
        .or(just('|').to(Operator::VerticalBar))
        .map(|p| Token::new(TokenKind::Operator(p), TokenValue::None));

    let control = select! {
        '{' => Control::BraceLeft,
        '}' => Control::BraceRight,
        '(' => Control::ParenLeft,
        ')' => Control::ParenRight,
        '[' => Control::BracketLeft,
        ']' => Control::BracketRight,
        '<' => Control::ChevronLeft,
        '>' => Control::ChevronRight,
        ';' => Control::Semicolon,
        ',' => Control::Comma,
        '.' => Control::FullStop,
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

    let line_annotation = just('#').ignore_then(take_until(newline)).map(|(c, n)| {
        Token::new(
            TokenKind::Annotation(Annotation::Line),
            TokenValue::String(c.into_iter().chain(n.into_iter()).collect()),
        )
    });

    let inline_annotation = just('`')
        .ignore_then(filter(|c| *c != '`').repeated())
        .then_ignore(just('`'))
        .collect()
        .map(|a| {
            Token::new(
                TokenKind::Annotation(Annotation::Inline),
                TokenValue::String(a),
            )
        });

    let annotation = line_annotation.or(inline_annotation);

    let token = space
        .or(comment)
        .or(annotation)
        .or(control)
        .or(operator)
        .or(property)
        .or(literal)
        .or(path_element)
        .or(keyword)
        .or(ident)
        .recover_with(skip_then_retry_until([]));

    token
        .map_with_span(|t, s| (t, s))
        .repeated()
        .then_ignore(end())
}
