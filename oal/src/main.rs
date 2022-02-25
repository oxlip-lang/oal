use oal_codegen::Builder;
use oal_compiler::{compile, Env, TagSeq, Transform, TypeConstrained, TypeConstraint};
use oal_syntax::ast::{Doc, Expr, Rel, Res, Stmt, TypedExpr};
use oal_syntax::parse;
use std::env;

fn relations(mut doc: Doc) -> oal_compiler::Result<Vec<Rel>> {
    let seq = &mut TagSeq::new();
    let env = &mut Env::new();

    doc.tag_type(seq, env)?;

    let cnt = &mut TypeConstraint::new();

    doc.constrain(cnt);

    let subst = &cnt.unify()?;

    doc.substitute(subst);

    let env = &mut Env::new();

    let doc = doc.transform(env, compile)?;

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
