use logos::Logos;
use oal_model::lexicon::{Intern, Interner, Lexeme, ParserError, Symbol, TokenList};
use oal_model::locator::Locator;
use oal_model::span::Span;

use crate::atom;

#[derive(Logos, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[logos(subpattern ident = r"[0-9a-zA-Z$_-]")]
pub enum TokenKind {
    #[regex(r"[ \t\r\n]+")]
    Space,
    #[regex(r"//[^\r\n]*[\r\n]*")]
    CommentLine,
    #[regex(r"/\*([^*]|\*[^/])*\*/")]
    CommentBlock,
    #[token("num")]
    PrimitiveNum,
    #[token("str")]
    PrimitiveStr,
    #[token("uri")]
    PrimitiveUri,
    #[token("bool")]
    PrimitiveBool,
    #[token("int")]
    PrimitiveInt,
    #[token("/")]
    PathElementRoot,
    #[regex("/[0-9a-zA-Z%~_.-]+")]
    PathElementSegment,
    #[token("get")]
    MethodGet,
    #[token("put")]
    MethodPut,
    #[token("post")]
    MethodPost,
    #[token("patch")]
    MethodPatch,
    #[token("delete")]
    MethodDelete,
    #[token("options")]
    MethodOptions,
    #[token("head")]
    MethodHead,
    #[token("media")]
    ContentMedia,
    #[token("headers")]
    ContentHeaders,
    #[token("status")]
    ContentStatus,
    #[token("let")]
    KeywordLet,
    #[token("res")]
    KeywordRes,
    #[token("use")]
    KeywordUse,
    #[token("as")]
    KeywordAs,
    #[token("on")]
    KeywordOn,
    #[token("rec")]
    KeywordRec,
    #[regex("[a-zA-Z_](?&ident)*")]
    IdentifierValue,
    #[regex("@(?&ident)+")]
    IdentifierReference,
    #[regex("[0-9]+")]
    LiteralNumber,
    #[regex("\"[^\"]*\"")]
    LiteralString,
    #[regex("[1-5]XX")]
    LiteralHttpStatus,
    #[regex("'[0-9a-zA-Z$@_-]+")]
    Property,
    #[token("{")]
    ControlBraceLeft,
    #[token("}")]
    ControlBraceRight,
    #[token("(")]
    ControlParenLeft,
    #[token(")")]
    ControlParenRight,
    #[token("[")]
    ControlBracketLeft,
    #[token("]")]
    ControlBracketRight,
    #[token("<")]
    ControlChevronLeft,
    #[token(">")]
    ControlChevronRight,
    #[token(";")]
    ControlSemicolon,
    #[token(".")]
    ControlFullStop,
    #[token(",")]
    ControlComma,
    #[token("!")]
    OperatorExclamationMark,
    #[token("?")]
    OperatorQuestionMark,
    #[token("&")]
    OperatorAmpersand,
    #[token("~")]
    OperatorTilde,
    #[token("|")]
    OperatorVerticalBar,
    #[token("=")]
    OperatorEqual,
    #[token(":")]
    OperatorColon,
    #[token("::")]
    OperatorDoubleColon,
    #[token("->")]
    OperatorArrow,
    #[regex(r"#[^\r\n]*[\r\n]*")]
    AnnotationLine,
    #[regex("`[^`]*`")]
    AnnotationInline,
}

#[test]
fn test_lexer() {
    let cases = [
        ("// comment", TokenKind::CommentLine),
        ("/* comment */", TokenKind::CommentBlock),
        ("\"string\"", TokenKind::LiteralString),
        ("499", TokenKind::LiteralNumber),
        ("4XX", TokenKind::LiteralHttpStatus),
        ("'prop", TokenKind::Property),
        ("@ref", TokenKind::IdentifierReference),
        ("val", TokenKind::IdentifierValue),
        (" \t\r\n", TokenKind::Space),
        ("`annotation`", TokenKind::AnnotationInline),
        ("# annotation", TokenKind::AnnotationLine),
        ("/", TokenKind::PathElementRoot),
        ("/abc", TokenKind::PathElementSegment),
    ];

    for (input, token) in cases {
        let mut lex = TokenKind::lexer(input);
        let t = lex
            .next()
            .expect("should return a result")
            .expect("should match a token");
        assert_eq!(t, token);
        assert_eq!(
            lex.slice().len(),
            input.len(),
            "input not fully matched: {:?}",
            input
        );
    }
}

// TODO: check if we need those predicates
impl TokenKind {
    pub fn is_comment(&self) -> bool {
        matches!(self, TokenKind::CommentLine | TokenKind::CommentBlock)
    }
    pub fn is_trivia(&self) -> bool {
        self.is_comment() || *self == TokenKind::Space
    }
    pub fn is_identifier(&self) -> bool {
        matches!(
            self,
            TokenKind::IdentifierReference | TokenKind::IdentifierValue
        )
    }
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            TokenKind::PrimitiveBool
                | TokenKind::PrimitiveInt
                | TokenKind::PrimitiveNum
                | TokenKind::PrimitiveStr
                | TokenKind::PrimitiveUri
        )
    }
    pub fn is_path_element(&self) -> bool {
        matches!(
            self,
            TokenKind::PathElementRoot | TokenKind::PathElementSegment
        )
    }
    pub fn is_method(&self) -> bool {
        matches!(
            self,
            TokenKind::MethodGet
                | TokenKind::MethodPut
                | TokenKind::MethodPost
                | TokenKind::MethodPatch
                | TokenKind::MethodDelete
                | TokenKind::MethodOptions
                | TokenKind::MethodHead
        )
    }
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            TokenKind::LiteralHttpStatus | TokenKind::LiteralNumber | TokenKind::LiteralString
        )
    }
    pub fn is_content(&self) -> bool {
        matches!(
            self,
            TokenKind::ContentHeaders | TokenKind::ContentMedia | TokenKind::ContentStatus
        )
    }
    pub fn is_operator(&self) -> bool {
        matches!(
            self,
            TokenKind::OperatorExclamationMark
                | TokenKind::OperatorQuestionMark
                | TokenKind::OperatorAmpersand
                | TokenKind::OperatorTilde
                | TokenKind::OperatorVerticalBar
                | TokenKind::OperatorEqual
                | TokenKind::OperatorColon
                | TokenKind::OperatorDoubleColon
                | TokenKind::OperatorArrow
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TokenValue {
    None,
    HttpStatus(atom::HttpStatus),
    Number(u64),
    Symbol(Symbol),
}

impl Intern for TokenValue {
    fn copy<I: Interner>(&self, from: &I, to: &mut I) -> Self {
        match self {
            TokenValue::Symbol(sym) => TokenValue::Symbol(to.register(from.resolve(*sym))),
            v => v.clone(),
        }
    }

    fn as_str<'a, I: Interner>(&'a self, from: &'a I) -> &'a str {
        match self {
            TokenValue::Symbol(sym) => from.resolve(*sym),
            _ => panic!("not a string"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenIdx;

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
        self.0.is_trivia()
    }
}

pub type TokenAlias = (TokenKind, TokenIdx);

fn parse_http_status(input: &str) -> atom::HttpStatus {
    let r = match input.chars().next().expect("should not be empty") {
        '1' => atom::HttpStatusRange::Info,
        '2' => atom::HttpStatusRange::Success,
        '3' => atom::HttpStatusRange::Redirect,
        '4' => atom::HttpStatusRange::ClientError,
        '5' => atom::HttpStatusRange::ServerError,
        _ => unreachable!("should be a valid http range"),
    };
    atom::HttpStatus::Range(r)
}

#[test]
fn test_parse_http_status() {
    assert_eq!(
        parse_http_status("4XX"),
        atom::HttpStatus::Range(atom::HttpStatusRange::ClientError)
    );
}

fn parse_number(input: &str) -> u64 {
    input.parse().expect("should be an unsigned integer")
}

fn parse_quoted_string(input: &str) -> &str {
    let len = input.len();
    assert!(len >= 2, "should be a quoted string");
    &input[1..len - 1]
}

#[test]
fn test_parse_quoted_string() {
    assert_eq!(parse_quoted_string("\"string\""), "string");
    assert_eq!(parse_quoted_string("`string`"), "string");
}

fn parse_prefixed_string(input: &str) -> &str {
    assert!(!input.is_empty(), "should be a prefixed string");
    &input[1..]
}

#[test]
fn test_prefixed_string() {
    assert_eq!(parse_prefixed_string("# annotation"), " annotation");
    assert_eq!(parse_prefixed_string("/segment"), "segment");
    assert_eq!(parse_prefixed_string("'prop"), "prop");
}

/// Parses a string of characters, yields a list of tokens and/or errors.
pub fn tokenize(loc: Locator, input: &str) -> (Option<TokenList<Token>>, Vec<ParserError>) {
    let lexer = TokenKind::lexer(input).spanned();
    let mut list = TokenList::new(loc.clone());
    let mut errors = Vec::new();

    for (result, range) in lexer {
        let span = Span::new(loc.clone(), range.clone());
        match result {
            Ok(kind) => {
                let slice = &input[range];
                let value = match kind {
                    TokenKind::LiteralNumber => TokenValue::Number(parse_number(slice)),
                    TokenKind::LiteralString => {
                        TokenValue::Symbol(list.register(parse_quoted_string(slice)))
                    }
                    TokenKind::LiteralHttpStatus => {
                        TokenValue::HttpStatus(parse_http_status(slice))
                    }
                    TokenKind::AnnotationLine => {
                        TokenValue::Symbol(list.register(parse_prefixed_string(slice)))
                    }
                    TokenKind::AnnotationInline => {
                        TokenValue::Symbol(list.register(parse_quoted_string(slice)))
                    }
                    TokenKind::IdentifierReference => TokenValue::Symbol(list.register(slice)),
                    TokenKind::IdentifierValue => TokenValue::Symbol(list.register(slice)),
                    TokenKind::PathElementSegment => {
                        TokenValue::Symbol(list.register(parse_prefixed_string(slice)))
                    }
                    TokenKind::Property => {
                        TokenValue::Symbol(list.register(parse_prefixed_string(slice)))
                    }
                    TokenKind::Space => TokenValue::Symbol(list.register(slice)),
                    _ => TokenValue::None,
                };
                let token = Token(kind, value);
                list.push((token, span));
            }
            Err(_) => errors.push(ParserError::new(span)),
        }
    }

    (Some(list), errors)
}

#[test]
fn test_tokenize() {
    let loc = Locator::try_from("file:///example.oal").unwrap();
    let input = "let @var = { 'p 100, 'p { 'p { 'p { 'p { 'p \"string\" ! }}}}};";

    let (Some(list), errors) = tokenize(loc, input) else { panic!() };

    assert!(errors.is_empty());
    assert_eq!(list.len(), input.len());
}
