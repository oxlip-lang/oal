pub mod ast;
pub mod errors;
mod parser;

#[cfg(test)]
mod ast_tests;

pub use self::parser::Parser;
pub use self::parser::Rule;

pub type Pair<'a> = pest::iterators::Pair<'a, Rule>;

pub fn parse(input: String) -> errors::Result<ast::Doc> {
    use pest::Parser as PestParser;

    let mut ast = Parser::parse(Rule::doc, &input)?;

    Ok(ast.next().unwrap().into())
}
