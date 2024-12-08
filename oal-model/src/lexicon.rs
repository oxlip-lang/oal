use crate::locator::Locator;
use crate::span::Span;
use generational_token_list::ItemToken;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::Range;
use string_interner::{DefaultSymbol, StringInterner};

pub type Symbol = DefaultSymbol;

pub trait Interner {
    fn register<T: AsRef<str>>(&mut self, s: T) -> Symbol;
    fn resolve(&self, sym: Symbol) -> &str;
}

pub trait Intern {
    fn copy<I: Interner>(&self, from: &I, to: &mut I) -> Self;
    fn as_str<'a, I: Interner>(&'a self, from: &'a I) -> &'a str;
}

pub trait Lexeme: Clone + PartialEq + Eq + Hash + Debug {
    type Kind: Copy + Clone + PartialEq + Eq + Hash + Debug;
    type Value: Debug + Intern;

    fn new(kind: Self::Kind, value: Self::Value) -> Self;
    fn is_trivia(kind: Self::Kind) -> bool;
    fn kind(&self) -> Self::Kind;
    fn value(&self) -> &Self::Value;
}

/// A cheap pointer to a token that retains the token kind.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TokenAlias<L: Lexeme>(L::Kind, ItemToken);

impl<L: Lexeme> Copy for TokenAlias<L> {}

impl<L: Lexeme> TokenAlias<L> {
    pub fn kind(&self) -> L::Kind {
        self.0
    }

    pub fn cursor(&self) -> Cursor {
        Cursor(Some(self.1))
    }
}

impl<L: Lexeme> Display for TokenAlias<L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.0)
    }
}

type ListArena<L> = generational_token_list::GenerationalTokenList<(L, Range<usize>)>;

/// A cursor in the stream of tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cursor(Option<ItemToken>);

impl Cursor {
    pub fn is_valid(&self) -> bool {
        self.0.is_some()
    }
}

pub struct TokenList<L: Lexeme> {
    arena: ListArena<L>,
    dict: StringInterner<string_interner::DefaultBackend>,
    loc: Locator,
}

impl<L: Lexeme> Interner for TokenList<L> {
    fn register<T: AsRef<str>>(&mut self, s: T) -> Symbol {
        self.dict.get_or_intern(s)
    }

    fn resolve(&self, sym: Symbol) -> &str {
        self.dict.resolve(sym).unwrap()
    }
}

impl<L> TokenList<L>
where
    L: Lexeme,
{
    pub fn new(loc: Locator) -> Self {
        TokenList {
            arena: ListArena::default(),
            dict: StringInterner::default(),
            loc,
        }
    }

    pub fn locator(&self) -> &Locator {
        &self.loc
    }

    pub fn reference(&self, s: Cursor) -> TokenRef<L> {
        TokenRef {
            list: self,
            token: s.0.expect("cursor should be valid"),
        }
    }

    pub fn head(&self) -> Cursor {
        Cursor(self.arena.head_token())
    }

    pub fn advance(&self, s: Cursor) -> Cursor {
        Cursor(s.0.and_then(|id| self.arena.next_token(id)))
    }

    pub fn kind(&self, s: Cursor) -> L::Kind {
        let id = s.0.expect("cursor should be valid");
        self.arena.get(id).unwrap().0.kind()
    }

    pub fn alias(&self, s: Cursor) -> TokenAlias<L> {
        let id = s.0.expect("cursor should be valid");
        TokenAlias(self.arena.get(id).unwrap().0.kind(), id)
    }

    pub fn token_span(&self, s: Cursor) -> (&L, Span) {
        let id = s.0.expect("cursor should be valid");
        let (token, range) = self.arena.get(id).unwrap();
        (token, Span::new(self.loc.clone(), range.clone()))
    }

    pub fn push(&mut self, token: L, range: Range<usize>) -> Cursor {
        Cursor(Some(self.arena.push_back((token, range))))
    }

    pub fn end(&self) -> usize {
        self.arena.tail().map_or(0, |(_, range)| range.end)
    }

    pub fn len(&self) -> usize {
        self.arena.len()
    }

    pub fn is_empty(&self) -> bool {
        self.arena.is_empty()
    }
}

impl<L: Lexeme> Debug for TokenList<L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = self.head();
        while s.is_valid() {
            let (token, span) = self.token_span(s);
            writeln!(f, "{:?} at {:?}", token, span)?;
            s = self.advance(s);
        }
        Ok(())
    }
}

pub struct TokenRef<'a, L: Lexeme> {
    list: &'a TokenList<L>,
    token: ItemToken,
}

impl<'a, L: Lexeme> TokenRef<'a, L> {
    pub fn token(&self) -> &'a L {
        &self.list.arena.get(self.token).unwrap().0
    }

    pub fn span(&self) -> Span {
        Span::new(
            self.list.loc.clone(),
            self.list.arena.get(self.token).unwrap().1.clone(),
        )
    }

    pub fn value(&self) -> &'a L::Value {
        self.token().value()
    }

    pub fn kind(&self) -> L::Kind {
        self.token().kind()
    }
}

impl<L: Lexeme> Debug for TokenRef<'_, L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at {:?}", self.token(), self.span())
    }
}

/// The tokenizer error type.
#[derive(Debug)]
pub struct ParserError(Span);

impl ParserError {
    pub fn new(span: Span) -> Self {
        ParserError(span)
    }

    pub fn span(&self) -> Span {
        self.0.clone()
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error at {}", self.0)
    }
}

impl std::error::Error for ParserError {}
