use crate::errors::Result;
use chumsky::{prelude::*, Stream};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use string_interner::{DefaultSymbol, StringInterner};

pub type Span = std::ops::Range<usize>;

pub type Symbol = DefaultSymbol;

pub trait Interner {
    fn register<T: AsRef<str>>(&mut self, s: T) -> Symbol;
    fn resolve(&self, sym: Symbol) -> &str;
}

pub trait Interned {
    fn copy<I: Interner>(&self, from: &I, to: &mut I) -> Self;
}

pub trait Lexeme: Clone + PartialEq + Eq + Hash + Debug {
    type Kind: Copy + Clone + PartialEq + Eq + Hash + Debug;
    type Value: Debug + Interned;

    fn new(kind: Self::Kind, value: Self::Value) -> Self;
    fn kind(&self) -> Self::Kind;
    fn value(&self) -> &Self::Value;
    fn is_trivia(&self) -> bool;
    fn internalize<I: Interner>(self, i: &mut I) -> Self;
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

type ListArena<L> = generational_token_list::GenerationalTokenList<(L, Span)>;

#[derive(Debug)]
pub struct TokenList<L: Lexeme> {
    list: ListArena<L>,
    dict: StringInterner,
}

impl<L: Lexeme> Default for TokenList<L> {
    fn default() -> Self {
        TokenList {
            list: ListArena::default(),
            dict: StringInterner::default(),
        }
    }
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
    pub fn get(&self, id: TokenIdx) -> &TokenSpan<L> {
        self.list.get(id).unwrap()
    }

    pub fn push(&mut self, t: TokenSpan<L>) -> TokenIdx {
        self.list.push_back(t)
    }

    pub fn len(&self) -> usize {
        self.list.tail().map_or(0, |(_, r)| r.end() + 1)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn stream<'a>(
        &'a self,
    ) -> Stream<TokenAlias<L>, Span, impl Iterator<Item = (TokenAlias<L>, Span)> + 'a> {
        let len = self.len();
        // Prepare the parser iterator by ignoring trivia tokens and replacing values by indices.
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
        Stream::from_iter(len..len + 1, iter)
    }
}

pub fn tokenize<L: Lexeme, P>(input: &str, lexer: P) -> Result<TokenList<L>>
where
    P: Parser<char, Vec<TokenSpan<L>>, Error = Simple<char>>,
{
    let mut token_list = TokenList::<L>::default();

    let (tokens, mut errs) = lexer.parse_recovery(input);

    if !errs.is_empty() {
        Err(errs.swap_remove(0).into())
    } else {
        if let Some(tokens) = tokens {
            // Note: Chumsky does not support stateful combinators at the moment.
            // Therefore we need a second pass over the vector of tokens to
            // internalize the strings and build the index list.
            tokens.into_iter().for_each(|(token, span)| {
                let new_token = token.internalize(&mut token_list);
                token_list.push((new_token, span));
            });
        }
        Ok(token_list)
    }
}
