use oal_codegen::Builder;
use oal_compiler::relations;
use oal_syntax::parse;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        panic!("missing input and output file parameters")
    }

    let input_file = &args[1];
    let output_file = &args[2];

    let input = std::fs::read_to_string(input_file).expect("reading failed");

    let ast = parse(input).expect("parsing failed");

    let rels = relations(&ast).expect("compilation failed");

    let api = Builder::new().expose_all(rels.iter()).open_api();

    let output = serde_yaml::to_string(&api).unwrap();

    std::fs::write(output_file, output).expect("writing failed");
}
