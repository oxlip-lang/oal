pub mod ast;
pub mod errors;
mod parser;

#[cfg(test)]
mod ast_tests;

pub use self::parser::Parser;
pub use self::parser::Rule;

use crate::ast::IntoNode;

pub type Pair<'a> = pest::iterators::Pair<'a, Rule>;

pub fn parse(input: String) -> errors::Result<ast::Program> {
    use pest::Parser as PestParser;

    let mut ast = Parser::parse(Rule::program, &input)?;

    Ok(ast.next().unwrap().into_node())
}
