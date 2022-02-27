use oal_codegen::Builder;
use oal_compiler::{
    compile, constrain, substitute, tag_type, Env, Scan, TagSeq, Transform, TypeConstraint,
};
use oal_syntax::ast::{Doc, Expr, Rel, Res, Stmt, TypedExpr};
use oal_syntax::parse;
use std::env;

fn relations(mut doc: Doc) -> oal_compiler::Result<Vec<Rel>> {
    doc.transform(&mut TagSeq::new(), &mut Env::new(), tag_type)?;

    let constraint = &mut TypeConstraint::new();

    doc.scan(constraint, &mut Env::new(), constrain)?;

    let subst = &mut constraint.unify()?;

    doc.transform(subst, &mut Env::new(), substitute)?;

    doc.transform(&mut (), &mut Env::new(), compile)?;

    doc.stmts
        .into_iter()
        .filter_map(|s| match s {
            Stmt::Res(Res {
                rel:
                    TypedExpr {
                        expr: Expr::Rel(r),
                        tag: _,
                    },
            }) => Some(Ok(r)),
            _ => None,
        })
        .collect()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        panic!("missing input and output file parameters")
    }

    let input_file = &args[1];
    let output_file = &args[2];

    let input = std::fs::read_to_string(input_file).expect("reading failed");

    let doc = parse(input).expect("parsing failed");

    let rels = relations(doc).expect("compilation failed");

    let api = Builder::new().expose_all(rels.iter()).open_api();

    let output = serde_yaml::to_string(&api).unwrap();

    std::fs::write(output_file, output).expect("writing failed");
}
