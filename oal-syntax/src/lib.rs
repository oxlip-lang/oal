pub mod ast;
mod parser;

pub use self::parser::Parser;
pub use self::parser::Rule;

pub type Pair<'a> = pest::iterators::Pair<'a, Rule>;

pub fn parse(input: &str) -> ast::Doc {
    use pest::Parser as PestParser;

    let input = std::fs::read_to_string(input).expect("cannot read file");
    Parser::parse(Rule::doc, &input)
        .expect("parsing failed")
        .next()
        .unwrap()
        .into()
}
