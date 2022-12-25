use super::lexer as lex;
use super::parser::{
    Application, Array, Content, Declaration, Gram, Literal, Object, PathElement, Primitive,
    Program, Property, Relation, Terminal, Transfer, UriSegment, UriTemplate, Variable, VariadicOp,
};
use crate::atom;
use oal_model::grammar::NodeRef;

type Prog<'a> = Program<'a, ()>;
type NRef<'a> = NodeRef<'a, (), Gram>;

fn parse<F: Fn(Prog)>(i: &str, f: F) {
    let tree = crate::rewrite::parse(i).expect("parsing failed");
    let prog = Program::cast(tree.root()).expect("expected a program");
    f(prog)
}

fn assert_decl<'a>(p: Prog<'a>, ident: &str) -> Declaration<'a, ()> {
    let decls = &mut p.declarations();
    let d = decls.next().expect("expected a declaration");
    assert!(decls.next().is_none(), "expected only one declaration");
    assert_eq!(d.ident(), ident);
    d
}

fn assert_term(n: NRef) -> NRef {
    let term = Terminal::cast(n).expect("expected a terminal");
    term.inner()
}

fn assert_prim(n: NRef, kind: lex::Primitive) -> Primitive<()> {
    let prim = Primitive::cast(n).expect("expected a primitive");
    assert_eq!(prim.primitive(), kind, "expected a type {:#?}", kind);
    prim
}

fn assert_next_path_elem<'a>(
    segs: &mut impl Iterator<Item = UriSegment<'a, ()>>,
) -> PathElement<'a, ()> {
    let UriSegment::Element(elem) = segs.next().expect("expected a segment") else { panic!("expected an URI element") };
    elem
}

fn assert_next_path_var<'a>(
    segs: &mut impl Iterator<Item = UriSegment<'a, ()>>,
) -> Property<'a, ()> {
    let UriSegment::Variable(var) = segs.next().expect("expected a segment") else { panic!("expected an URI variable") };
    Property::cast(assert_term(var.inner())).expect("expected a property")
}

fn assert_next_prop<'a>(
    props: &mut impl Iterator<Item = NodeRef<'a, (), Gram>>,
) -> Property<'a, ()> {
    let n = props.next().expect("expected a node");
    Property::cast(assert_term(n)).expect("expected a property")
}

fn assert_lit(n: NRef) -> Literal<()> {
    Literal::cast(n).expect("expected a literal")
}

#[test]
fn parse_decl_primitive() {
    parse("let a = num;", |p: Prog| {
        let rhs = assert_term(assert_decl(p, "a").rhs());
        assert_prim(rhs, lex::Primitive::Num);
    })
}

#[test]
fn parse_decl_ident() {
    parse("let a = b;", |p: Prog| {
        let rhs = assert_term(assert_decl(p, "a").rhs());
        let var = Variable::cast(rhs).expect("expected a variable");
        assert_eq!(var.ident(), "b");
    })
}

#[test]
fn parse_decl_array() {
    parse("let a = [str];", |p: Prog| {
        let rhs = assert_term(assert_decl(p, "a").rhs());
        let arr = Array::cast(rhs).expect("expected an array");
        assert_prim(assert_term(arr.inner()), lex::Primitive::Str);
    })
}

#[test]
fn parse_decl_uri() {
    parse("let a = /p;", |p: Prog| {
        let rhs = assert_term(assert_decl(p, "a").rhs());
        let segs = &mut UriTemplate::cast(rhs)
            .expect("expected an URI template")
            .segments();
        let UriSegment::Element(elem) = segs.next().expect("expected a segment") else { panic!("expected an URI element") };
        assert_eq!(elem.as_str(), "p");
    });
    parse("let a = /x/{ 'y str }/z?{ 'q str, 'n num };", |p: Prog| {
        let rhs = assert_term(assert_decl(p, "a").rhs());
        let uri = UriTemplate::cast(rhs).expect("expected an URI template");
        let segs = &mut uri.segments();

        assert_eq!(assert_next_path_elem(segs).as_str(), "x");

        let prop = assert_next_path_var(segs);
        assert_eq!(prop.name(), "y");
        assert_prim(assert_term(prop.rhs()), lex::Primitive::Str);

        assert_eq!(assert_next_path_elem(segs).as_str(), "z");

        let params = uri.params().expect("expected URI parameters");
        let props = &mut params.properties();

        let prop = assert_next_prop(props);
        assert_eq!(prop.name(), "q");
        assert_prim(assert_term(prop.rhs()), lex::Primitive::Str);

        let prop = assert_next_prop(props);
        assert_eq!(prop.name(), "n");
        assert_prim(assert_term(prop.rhs()), lex::Primitive::Num);
    })
}

#[test]
fn parse_decl_transfer() {
    parse("let a = get -> {};", |p: Prog| {
        let xfer = Transfer::cast(assert_decl(p, "a").rhs()).expect("expected transfer");

        let methods: Vec<_> = xfer.methods().collect();
        assert_eq!(methods, vec![atom::Method::Get]);

        assert!(xfer.params().is_none());
        assert!(xfer.domain().is_none());
        assert_term(xfer.range());
    });
    parse("let a = get, put { 'q str } : {} -> {};", |p: Prog| {
        let xfer = Transfer::cast(assert_decl(p, "a").rhs()).expect("expected transfer");

        let methods: Vec<_> = xfer.methods().collect();
        assert_eq!(methods, vec![atom::Method::Get, atom::Method::Put]);

        let params = xfer.params().expect("expected parameters");
        let props = &mut params.properties();

        let prop = assert_next_prop(props);
        assert_eq!(prop.name(), "q");
        assert_prim(assert_term(prop.rhs()), lex::Primitive::Str);

        assert!(xfer.domain().is_some(), "expected a domain");
        assert_term(xfer.range());
    });
    parse("let a = get -> <{}> :: <{}>;", |p: Prog| {
        let xfer = Transfer::cast(assert_decl(p, "a").rhs()).expect("expected transfer");
        let op = VariadicOp::cast(xfer.range()).expect("expected an operation");
        assert_eq!(op.operator(), atom::Operator::Range);

        let opds = &mut op.operands();
        Content::cast(assert_term(opds.next().expect("expected operand")))
            .expect("expected first content");
        Content::cast(assert_term(opds.next().expect("expected operand")))
            .expect("expected second content");
        assert!(opds.next().is_none(), "expected no more operand");
    })
}

#[test]
fn parse_decl_property() {
    parse("let a = 'q str;", |p: Prog| {
        let prop =
            Property::cast(assert_term(assert_decl(p, "a").rhs())).expect("expected a property");
        assert_eq!(prop.name(), "q");
        assert_prim(assert_term(prop.rhs()), lex::Primitive::Str);
    })
}

#[test]
fn parse_decl_number() {
    parse("let a = 404;", |p: Prog| {
        let lit = assert_lit(assert_term(assert_decl(p, "a").rhs()));
        assert_eq!(lit.kind(), lex::Literal::Number);
        let lex::TokenValue::Number(num) = lit.value() else { panic!("expected a number") };
        assert_eq!(*num, 404);
    });
    parse("let a = 4XX;", |p: Prog| {
        let lit = assert_lit(assert_term(assert_decl(p, "a").rhs()));
        assert_eq!(lit.kind(), lex::Literal::HttpStatus);
        let lex::TokenValue::HttpStatus(status) = lit.value() else { panic!("expected a status") };
        assert_eq!(
            *status,
            atom::HttpStatus::Range(atom::HttpStatusRange::ClientError)
        );
    })
}

#[test]
fn parse_decl_string() {
    parse(r#"let a = "application/json";"#, |p: Prog| {
        let lit = assert_lit(assert_term(assert_decl(p, "a").rhs()));
        assert_eq!(lit.kind(), lex::Literal::String);
        assert_eq!(lit.as_str(), "application/json");
    })
}

#[test]
fn parse_decl_reference() {
    parse("let @a = {};", |p: Prog| {
        let decl = assert_decl(p, "@a");
        assert!(decl.ident().is_reference());
    })
}

#[test]
fn parse_import() {
    parse(r#"use "module";"#, |p: Prog| {
        let imp = p.imports().next().expect("expected an import");
        assert_eq!(imp.module(), "module");
    })
}

#[test]
fn parse_decl_ann_inline() {
    parse(r#"let a = num `title: "number"`;"#, |p: Prog| {
        let term = Terminal::cast(assert_decl(p, "a").rhs()).expect("expected a terminal");
        assert_eq!(
            term.annotation().expect("expected an inline annotation"),
            r#"title: "number""#
        );
    });
    parse(r#"let a = num;"#, |p: Prog| {
        let term = Terminal::cast(assert_decl(p, "a").rhs()).expect("expected a terminal");
        assert!(term.annotation().is_none());
    })
}

#[test]
fn parse_decl_content() {
    parse(
        r#"let a = <media="application/json", status=200, headers={}, {}>;"#,
        |p: Prog| {
            let cnt =
                Content::cast(assert_term(assert_decl(p, "a").rhs())).expect("expected a content");

            let body = cnt.body().expect("expected a content body");
            Object::cast(assert_term(body)).expect("expected an object");

            let metas = &mut cnt.meta();

            let meta = metas.next().expect("expected meta");
            assert_eq!(meta.tag(), lex::Content::Media);
            assert_eq!(assert_term(meta.rhs()).as_str(), "application/json");

            let meta = metas.next().expect("expected meta");
            assert_eq!(meta.tag(), lex::Content::Status);
            let lex::TokenValue::Number(num) = assert_lit(assert_term(meta.rhs())).value() else { panic!("expected a number" )};
            assert_eq!(*num, 200);

            let meta = metas.next().expect("expected meta");
            assert_eq!(meta.tag(), lex::Content::Headers);
            Object::cast(assert_term(meta.rhs())).expect("expected an object");

            assert!(metas.next().is_none());
        },
    );
    parse(r#"let a = <status=204,>;"#, |p: Prog| {
        let cnt =
            Content::cast(assert_term(assert_decl(p, "a").rhs())).expect("expected a content");

        assert!(cnt.body().is_none());

        let metas = &mut cnt.meta();

        let meta = metas.next().expect("expected meta");
        assert_eq!(meta.tag(), lex::Content::Status);
        let lex::TokenValue::Number(num) = assert_lit(assert_term(meta.rhs())).value() else { panic!("expected a number" )};
        assert_eq!(*num, 204);

        assert!(metas.next().is_none());
    });
    parse(r#"let a = <>;"#, |p: Prog| {
        let cnt =
            Content::cast(assert_term(assert_decl(p, "a").rhs())).expect("expected a content");

        assert!(cnt.body().is_none());
        assert!(cnt.meta().next().is_none());
    })
}

#[test]
fn parse_decl_lambda() {
    parse("let f x y z = num;", |p: Prog| {
        let decl = assert_decl(p, "f");
        let bindings: Vec<_> = decl.bindings().map(|b| b.ident()).collect();
        assert_eq!(bindings, vec!["x", "y", "z"]);
        assert_prim(assert_term(decl.rhs()), lex::Primitive::Num);
    })
}

#[test]
fn parse_decl_application() {
    parse("let a = f num {} uri;", |p: Prog| {
        let decl = assert_decl(p, "a");

        let app = Application::cast(decl.rhs()).expect("expected an application");
        assert_eq!(app.ident(), "f");

        let arguments = &mut app.arguments();
        assert_prim(
            arguments.next().expect("expected an argument").inner(),
            lex::Primitive::Num,
        );
        Object::cast(arguments.next().expect("expected an argument").inner())
            .expect("expected an object");
        assert_prim(
            arguments.next().expect("expected an argument").inner(),
            lex::Primitive::Uri,
        );
        assert!(arguments.next().is_none(), "expected no more argument");
    })
}

#[test]
fn parse_decl_variadic_op() {
    parse("let a = {} ~ uri ~ bool;", |p: Prog| {
        let decl = assert_decl(p, "a");

        let op = VariadicOp::cast(decl.rhs()).expect("expected variadic operator");
        assert_eq!(op.operator(), atom::Operator::Any);

        let opds = &mut op.operands();
        Object::cast(assert_term(opds.next().expect("expected operand"))).expect("expected object");
        assert_prim(
            assert_term(opds.next().expect("expected operand")),
            lex::Primitive::Uri,
        );
        assert_prim(
            assert_term(opds.next().expect("expected operand")),
            lex::Primitive::Bool,
        );
        assert!(opds.next().is_none(), "expected no more operand");
    })
}

#[test]
fn parse_decl_relation() {
    parse("let a = /p ( put : <{}> -> <{}> );", |p: Prog| {
        let decl = assert_decl(p, "a");
        let rel = Relation::cast(decl.rhs()).expect("expected a relation");

        let uri = UriTemplate::cast(rel.uri().inner()).expect("expected an URI template");
        let segs = &mut uri.segments();
        let UriSegment::Element(elem) = segs.next().expect("expected an URI segment") else { panic!("expected path element") };
        assert_eq!(elem.as_str(), "p");
        assert!(segs.next().is_none(), "expected no more URI segment");

        let xfers = &mut rel.transfers();

        let xfer = Transfer::cast(xfers.next().expect("expected a transfer")).unwrap();
        let methods: Vec<_> = xfer.methods().collect();
        assert_eq!(methods, vec![atom::Method::Put]);

        assert!(xfers.next().is_none(), "expected no more transfer");
    });
    parse(
        r#"
let a = /p (
    patch, put : <{}> -> <{}>,
    get               -> <{}>
);
"#,
        |p: Prog| {
            let decl = assert_decl(p, "a");
            let rel = Relation::cast(decl.rhs()).expect("expected a relation");

            let uri = UriTemplate::cast(rel.uri().inner()).expect("expected an URI template");
            let segs = &mut uri.segments();
            let UriSegment::Element(elem) = segs.next().expect("expected an URI segment") else { panic!("expected path element") };
            assert_eq!(elem.as_str(), "p");
            assert!(segs.next().is_none(), "expected no more URI segment");

            let xfers = &mut rel.transfers();

            let xfer = Transfer::cast(xfers.next().expect("expected a transfer")).unwrap();
            let methods: Vec<_> = xfer.methods().collect();
            assert_eq!(methods, vec![atom::Method::Patch, atom::Method::Put]);

            let xfer = Transfer::cast(xfers.next().expect("expected a transfer")).unwrap();
            let methods: Vec<_> = xfer.methods().collect();
            assert_eq!(methods, vec![atom::Method::Get]);

            assert!(xfers.next().is_none(), "expected no more transfer");
        },
    );
    parse("let a = /p ( i, j );", |p: Prog| {
        let decl = assert_decl(p, "a");
        let rel = Relation::cast(decl.rhs()).expect("expected a relation");
        let xfers = &mut rel.transfers();
        assert_eq!(xfers.count(), 2);
    });
}

#[test]
fn parse_annotation() {
    parse(
        r#"
# description: "some identifier"
# required: true
let id = num;
# description: "some record"
let r = {};
"#,
        |p: Prog| {
            let decls = &mut p.declarations();

            let decl = decls.next().expect("expected a declaration");
            let anns = &mut decl.annotations();
            assert_eq!(
                anns.next().expect("expected an annotation"),
                " description: \"some identifier\"\n"
            );
            assert_eq!(
                anns.next().expect("expected an annotation"),
                " required: true\n"
            );
            assert!(anns.next().is_none(), "expected no more annotation");

            let decl = decls.next().expect("expected another declaration");
            let anns = &mut decl.annotations();
            assert_eq!(
                anns.next().expect("expected an annotation"),
                " description: \"some record\"\n"
            );
            assert!(anns.next().is_none(), "expected no more annotation");
        },
    )
}

#[test]
fn parse_resource() {
    parse("res / (get -> <>);", |p: Prog| {
        let res = p.resources().next().expect("expected a resource");
        let rel = Relation::cast(res.relation()).expect("expected a relation");
        UriTemplate::cast(rel.uri().inner()).expect("expected an URI template");
        rel.transfers().next().expect("expected a transfer");
    })
}

#[test]
fn parse_grammar_error() {
    let Err(err) = crate::rewrite::parse::<_, ()>("res / ( get -> );")
        else { panic!("expected an error") };
    let crate::errors::Error::Parser(err) = err
        else { panic!("expected a parser error") };
    assert!(
        matches!(*err, oal_model::errors::Error::Grammar(_)),
        "expected a grammar error"
    );
}

#[test]
fn parse_lexicon_error() {
    let Err(err) = crate::rewrite::parse::<_, ()>("!!! / ( get -> );")
        else { panic!("expected an error") };
    let crate::errors::Error::Parser(err) = err
        else { panic!("expected a parser error") };
    assert!(
        matches!(*err, oal_model::errors::Error::Lexicon(_)),
        "expected a lexicon error"
    );
}
