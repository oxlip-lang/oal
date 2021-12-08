use oal_codegen::Api;
use oal_compiler::relations;
use oal_syntax::parse;

fn main() {
    let input = std::fs::read_to_string("doc.txt").expect("reading failed");

    let ast = parse(input).expect("parsing failed");

    println!("{:#?}", ast);

    let rels = relations(&ast).expect("compilation failed");

    println!("{:#?}", rels);

    let api = Api::new().expose_all(rels.iter()).render();

    let output = serde_yaml::to_string(&api).unwrap();

    println!("{}", output);
}
