use crate::locator::Locator;
use crate::span::Span;
use chumsky::Stream;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
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
    fn kind(&self) -> Self::Kind;
    fn value(&self) -> &Self::Value;
    fn is_trivia(&self) -> bool;
}

pub type TokenIdx = generational_token_list::ItemToken;

pub type TokenSpan<L> = (L, Span);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TokenAlias<L: Lexeme>(L::Kind, TokenIdx);

impl<L: Lexeme> Copy for TokenAlias<L> {}

impl<L: Lexeme> TokenAlias<L> {
    pub fn new(kind: L::Kind, idx: TokenIdx) -> Self {
        TokenAlias(kind, idx)
    }

    pub fn kind(&self) -> L::Kind {
        self.0
    }

    pub fn index(&self) -> TokenIdx {
        self.1
    }
}

impl<L: Lexeme> Display for TokenAlias<L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.0)
    }
}

type ListArena<L> = generational_token_list::GenerationalTokenList<TokenSpan<L>>;

#[derive(Debug)]
pub struct TokenList<L: Lexeme> {
    list: ListArena<L>,
    dict: StringInterner,
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
            list: ListArena::default(),
            dict: StringInterner::default(),
            loc,
        }
    }

    pub fn locator(&self) -> &Locator {
        &self.loc
    }

    pub fn get(&self, id: TokenIdx) -> &TokenSpan<L> {
        self.list.get(id).unwrap()
    }

    pub fn push(&mut self, t: TokenSpan<L>) -> TokenIdx {
        self.list.push_back(t)
    }

    pub fn len(&self) -> usize {
        self.list.tail().map_or(0, |(_, r)| r.end())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[allow(clippy::needless_lifetimes)]
    pub fn stream<'a>(
        &'a self,
    ) -> Stream<TokenAlias<L>, Span, impl Iterator<Item = (TokenAlias<L>, Span)> + 'a> {
        let len = self.len();
        // Prepare the parser iterator by ignoring trivia tokens and replacing values with indices.
        let iter = self
            .list
            .iter_with_tokens()
            .filter_map(move |(index, (token, span))| {
                if token.is_trivia() {
                    None
                } else {
                    Some((TokenAlias::new(token.kind(), index), span.clone()))
                }
            });
        Stream::from_iter(Span::new(self.loc.clone(), len..len + 1), iter)
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
        write!(f, "error at {:?}", self.0)
    }
}

impl std::error::Error for ParserError {}
