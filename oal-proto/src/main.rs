use oal_syntax::rewrite::lexer::Lex;
use oal_syntax::rewrite::parser::{Gram, Program};

fn main() -> anyhow::Result<()> {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();

    let tokens = Lex::tokenize(src.as_str())?;
    let syntax = Gram::parse::<()>(tokens)?;

    if let Some(p) = Program::cast(syntax.root()) {
        for n in p.node().descendants() {
            println!("{:?}", n.syntax().trunk());
        }
    }

    Ok(())
}
