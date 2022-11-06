use crate::errors::Result;
use crate::rewrite::parser::Gram;
use oal_model::grammar::{analyze, SyntaxTree};
use oal_model::lexicon::tokenize;

pub mod lexer;
pub mod parser;

#[cfg(test)]
mod tests;

/// Perform lexical and syntax analysis, yielding a concrete syntax tree.
pub fn parse<I, T>(input: I) -> Result<SyntaxTree<T, Gram>>
where
    I: AsRef<str>,
    T: Clone + Default,
{
    let tokens = tokenize(input, lexer::lexer())?;
    let syntax = analyze::<_, _, T>(tokens, parser::parser())?;

    Ok(syntax)
}
