pub mod atom;
pub mod errors;
pub mod lexer;
pub mod parser;

#[cfg(test)]
mod tests;

use crate::errors::Result;
use crate::parser::Gram;
use oal_model::grammar::{analyze, Core, SyntaxTree};
use oal_model::lexicon::tokenize;
use oal_model::locator::Locator;

/// Perform lexical and syntax analysis, yielding a concrete syntax tree.
pub fn parse<I: AsRef<str>, T: Core>(loc: Locator, input: I) -> Result<SyntaxTree<T, Gram>> {
    let tokens = tokenize(loc, input, lexer::lexer())?;
    let syntax = analyze::<_, _, T>(tokens, parser::parser())?;

    Ok(syntax)
}
