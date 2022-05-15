pub mod ast;
pub mod errors;
mod parser;

#[cfg(test)]
mod ast_tests;

pub use self::parser::Parser;
pub use self::parser::Rule;

use crate::ast::IntoExpr;

pub type Pair<'a> = pest::iterators::Pair<'a, Rule>;

pub fn parse<T: ast::AsExpr, I: AsRef<str>>(input: I) -> errors::Result<ast::Program<T>> {
    use pest::Parser as PestParser;

    let mut pairs = Parser::parse(Rule::program, input.as_ref())?;

    Ok(pairs.next().unwrap().into_expr())
}
