pub mod atom;
pub mod errors;
pub mod lexer;
pub mod parser;

#[cfg(test)]
mod tests;

use crate::errors::Error;
use crate::parser::Gram;
use oal_model::grammar::{Context, Core, ParserError, ParserMatch, SyntaxTree};
use oal_model::locator::Locator;

/// Performs lexical and syntax analysis, yields a concrete syntax tree.
pub fn parse<I: AsRef<str>, T: Core>(
    loc: Locator,
    input: I,
) -> (Option<SyntaxTree<T, Gram>>, Vec<Error>) {
    let (tokens, lex_errs) = crate::lexer::tokenize(loc, input.as_ref());
    let mut errs = lex_errs.into_iter().map(Error::from).collect::<Vec<_>>();
    if let Some(tokens) = tokens {
        let mut ctx = Context::new(tokens);
        let cursor = ctx.head();
        match crate::parser::parse_program(&mut ctx, cursor) {
            Ok((s, root)) => {
                if s.is_valid() {
                    errs.push(ParserError::new("cannot parse remaining input", ctx.span(s)).into());
                }
                match root {
                    ParserMatch::Node(n) => (Some(ctx.tree().finalize(n)), errs),
                    _ => (None, errs),
                }
            }
            Err(err) => {
                errs.push(Error::from(err));
                (None, errs)
            }
        }
    } else {
        (None, errs)
    }
}
