use crate::atom;
use logos::Logos;
use oal_model::lexicon::{Intern, Interner, Lexeme, ParserError, Symbol, TokenList};
use oal_model::locator::Locator;
use oal_model::span::Span;

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
    // Produced contextually after a URI parameter, without a new slash.
    PathElementSuffix,
    PathVariableSuffix,
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
            TokenKind::PathElementRoot
                | TokenKind::PathElementSegment
                | TokenKind::PathElementSuffix
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

    fn is_trivia(kind: TokenKind) -> bool {
        kind.is_trivia()
    }
}

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
    let mut lexer = TokenKind::lexer(input);
    let mut list = TokenList::new(loc.clone());
    let mut errors = Vec::new();
    let mut path: Option<(usize, bool)> = None;
    let mut brace_depth = 0_usize;
    let mut uri_variables = Vec::new();

    loop {
        let offset = lexer.span().end;
        let remainder = lexer.remainder();
        // Keep operators and the relation keyword as boundaries. Only adjacent
        // fragments join a segment; whitespace still separates OAL expressions.
        let relation = remainder.strip_prefix("on").is_some_and(|rest| {
            !rest.starts_with(|c: char| c.is_ascii_alphanumeric() || "$_-".contains(c))
        });
        let suffix_len = if path.is_some_and(|(end, _)| end == offset)
            && !remainder.starts_with("->")
            && !relation
        {
            remainder
                .bytes()
                .take_while(|c| c.is_ascii_alphanumeric() || b"%_.-".contains(c))
                .count()
        } else {
            0
        };
        if suffix_len > 0 {
            lexer.bump(suffix_len);
            let end = offset + suffix_len;
            let value = TokenValue::Symbol(list.register(&input[offset..end]));
            list.push(Token(TokenKind::PathElementSuffix, value), offset..end);
            path = Some((end, false));
            continue;
        }
        let variable = path
            .filter(|(end, root)| *root || *end == offset)
            .filter(|_| remainder.starts_with('{'));
        let Some(result) = lexer.next() else { break };
        let range = lexer.span();
        match result {
            Ok(mut kind) => {
                match kind {
                    TokenKind::PathElementRoot | TokenKind::PathElementSegment => {
                        path = Some((range.end, kind == TokenKind::PathElementRoot));
                    }
                    TokenKind::ControlBraceLeft => {
                        brace_depth += 1;
                        if let Some((_, root)) = variable {
                            uri_variables.push(brace_depth);
                            if !root {
                                kind = TokenKind::PathVariableSuffix;
                            }
                        }
                        path = None;
                    }
                    TokenKind::ControlBraceRight => {
                        path = if uri_variables.last() == Some(&brace_depth) {
                            uri_variables.pop();
                            Some((range.end, false))
                        } else {
                            None
                        };
                        brace_depth = brace_depth.saturating_sub(1);
                    }
                    _ if kind.is_trivia() && path.is_some_and(|(_, root)| root) => {}
                    _ => path = None,
                }
                let slice = &input[range.clone()];
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
                    TokenKind::CommentBlock | TokenKind::CommentLine | TokenKind::Space => {
                        TokenValue::Symbol(list.register(slice))
                    }
                    _ => TokenValue::None,
                };
                let token = Token(kind, value);
                list.push(token, range);
            }
            Err(_) => {
                path = None;
                let span = Span::new(loc.clone(), range);
                errors.push(ParserError::new(span));
            }
        }
    }

    (Some(list), errors)
}

#[test]
fn test_tokenize() {
    let loc = Locator::try_from("file:///example.oal").unwrap();
    let input = "let @var = { 'p 100, 'p { 'p { 'p { 'p { 'p \"string\" ! }}}}};";

    let (Some(list), errors) = tokenize(loc, input) else {
        panic!()
    };

    assert!(errors.is_empty());
    assert_eq!(list.end(), input.len());
}

#[test]
fn tokenize_mixed_segments_only_reclassifies_adjacent_fragments() {
    let input = "/v{'id int}.json/{'name str}get/{'n int}123456789012345678901234567890";
    let (list, errors) = tokenize(Locator::try_from("file:///test.oal").unwrap(), input);
    assert!(errors.is_empty());
    let list = list.unwrap();
    let mut cursor = list.head();
    let mut suffixes = Vec::new();
    while cursor.is_valid() {
        let (token, span) = list.token_span(cursor);
        if token.kind() == TokenKind::PathElementSuffix {
            let text = token.value().as_str(&list);
            assert_eq!(text, &input[span.range()]);
            suffixes.push(text);
        }
        cursor = list.advance(cursor);
    }
    assert_eq!(suffixes, [".json", "get", "123456789012345678901234567890"]);
}

#[test]
fn uri_fragment_state_does_not_escape_nested_braces_or_comments() {
    let input = "/v{f {'value \"}\", 'nested /x/{'id int}.json}}.txt /* { */ mod.name ~ str -> int";
    let (list, errors) = tokenize(Locator::try_from("file:///test.oal").unwrap(), input);
    assert!(errors.is_empty());
    let list = list.unwrap();
    let mut cursor = list.head();
    let mut suffixes = Vec::new();
    let mut dots = 0;
    while cursor.is_valid() {
        let token = list.reference(cursor);
        if token.kind() == TokenKind::PathElementSuffix {
            suffixes.push(token.value().as_str(&list));
        }
        if token.kind() == TokenKind::ControlFullStop {
            dots += 1;
        }
        cursor = list.advance(cursor);
    }
    assert_eq!(suffixes, [".json", ".txt"]);
    assert_eq!(dots, 1);
}
