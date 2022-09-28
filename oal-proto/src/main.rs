use oal_model::grammar::NodeIdx;
use oal_syntax::rewrite::lexer::Lex;
use oal_syntax::rewrite::parser::{Gram, Program, Resource};

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();

    let tokens = Lex::tokenize(src.as_str());
    let syntax = Gram::parse(tokens);

    let mut index = Option::<NodeIdx>::None;

    if let Some(p) = Program::cast(syntax.root()) {
        assert_eq!(*p.node().syntax().core_ref(), 0);

        for n in p.node().descendants() {
            println!("{:?}", n.syntax().trunk());
            if let Some(r) = Resource::cast(n) {
                println!("Resource: {:?}", r.node().syntax().trunk());
                r.node().syntax().core_mut(|mut c| *c = 42);
                index = Some(r.node().index());

                let n2 = r.node().detach();
                println!("Detached tree: {:#?}", n2);
            }
        }

        println!(
            "Updated core: {:#?}",
            p.resources().next().unwrap().node().syntax().core_ref()
        );
    }

    if let Some(_idx) = index {
        // let i = list.push_back(...);
        // let token = SyntaxToken(TokenKind::Literal(Literal::Number), i);
        // tree.new_node(SyntaxNode::new(SyntaxTrunk::Leaf(token), 0));
        // idx.insert_before(n, &mut tree);
        // idx.remove_subtree(&mut tree);
    }
}
