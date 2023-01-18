pub mod atom;
pub mod errors;
pub mod lexer;
pub mod parser;

#[cfg(test)]
mod tests;

use crate::errors::Error;
use crate::parser::Gram;
use oal_model::grammar::{analyze, Core, SyntaxTree};
use oal_model::lexicon::tokenize;
use oal_model::locator::Locator;

/// Perform lexical and syntax analysis, yielding a concrete syntax tree.
pub fn parse<I: AsRef<str>, T: Core>(
    loc: Locator,
    input: I,
) -> (Option<SyntaxTree<T, Gram>>, Vec<Error>) {
    let (tokens, lex_errs) = tokenize(loc, input, lexer::lexer());
    let errs = lex_errs.into_iter().map(Error::from);
    if let Some(tokens) = tokens {
        let (tree, syn_errs) = analyze::<_, _, T>(tokens, parser::parser());
        let errs = errs.chain(syn_errs.into_iter().map(Error::from));
        (tree, errs.collect())
    } else {
        (None, errs.collect())
    }
}
