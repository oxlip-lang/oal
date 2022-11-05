use oal_model::grammar::parse;
use oal_model::lexicon::tokenize;
use oal_syntax::rewrite::lexer::lexer;
use oal_syntax::rewrite::parser::{parser, Program};

fn main() -> anyhow::Result<()> {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();

    let tokens = tokenize(src.as_str(), lexer())?;
    let syntax = parse::<_, _, ()>(tokens, parser())?;

    if let Some(p) = Program::cast(syntax.root()) {
        for n in p.node().descendants() {
            println!("{:?}", n.syntax().trunk());
        }
    }

    Ok(())
}
