use crate::rewrite::lexer as lex;
use crate::rewrite::parser::{
    Array, Declaration, Gram, Primitive, Program, Property, Symbol, Terminal, Transfer, UriPath,
    UriSegment, UriTemplate,
};
use oal_model::grammar::NodeRef;

use super::parser::PathElement;

fn parse<F: Fn(Program<()>)>(i: &str, f: F) {
    let tree = crate::rewrite::parse(i).expect("parsing failed");
    let prog = Program::cast(tree.root()).expect("expected a program");
    f(prog)
}

fn assert_decl<'a>(p: Program<'a, ()>, sym: &str) -> Declaration<'a, ()> {
    assert_eq!(p.declarations().count(), 1, "expected one declaration");
    let d = p.declarations().next().unwrap();
    assert_eq!(d.symbol().as_ident().as_ref(), sym);
    d
}

fn assert_term(n: NodeRef<(), Gram>) -> NodeRef<(), Gram> {
    let term = Terminal::cast(n).expect("expected a terminal");
    term.inner()
}

fn assert_prim(n: NodeRef<(), Gram>, kind: lex::Primitive) -> Primitive<()> {
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

#[test]
fn parse_decl_primitive() {
    parse("let a = num;", |p: Program<()>| {
        let rhs = assert_term(assert_decl(p, "a").rhs());
        assert_prim(rhs, lex::Primitive::Num);
    })
}

#[test]
fn parse_decl_symbol() {
    parse("let a = b;", |p: Program<()>| {
        let rhs = assert_term(assert_decl(p, "a").rhs());
        let sym = Symbol::cast(rhs).expect("expected a symbol");
        assert_eq!(sym.as_ident().as_ref(), "b");
    })
}

#[test]
fn parse_decl_array() {
    parse("let a = [str];", |p: Program<()>| {
        let rhs = assert_term(assert_decl(p, "a").rhs());
        let arr = Array::cast(rhs).expect("expected an array");
        assert_prim(assert_term(arr.inner()), lex::Primitive::Str);
    })
}

#[test]
fn parse_decl_uri() {
    parse("let a = /;", |p: Program<()>| {
        let rhs = assert_term(assert_decl(p, "a").rhs());
        let tmpl = UriTemplate::cast(rhs).expect("expected an URI template");
        let uri = UriPath::cast(tmpl.path()).expect("expected an URI path");
        let mut segs = uri.segments();
        let UriSegment::Element(elem) = segs.next().expect("expected a segment") else { panic!("expected an URI element") };
        assert_eq!(elem.as_str(), "/");
    });
    parse(
        "let a = /x/{ 'y str }/z?{ 'q str, 'n num };",
        |p: Program<()>| {
            let rhs = assert_term(assert_decl(p, "a").rhs());
            let tmpl = UriTemplate::cast(rhs).expect("expected an URI template");
            let uri = UriPath::cast(tmpl.path()).expect("expected an URI path");
            let segs = &mut uri.segments();

            assert_eq!(assert_next_path_elem(segs).as_str(), "x");
            assert_eq!(assert_next_path_elem(segs).as_str(), "/");

            let prop = assert_next_path_var(segs);
            assert_eq!(prop.name().as_ident().as_ref(), "y");
            assert_prim(assert_term(prop.rhs()), lex::Primitive::Str);

            assert_eq!(assert_next_path_elem(segs).as_str(), "z");

            let params = tmpl.params().expect("expected URI paramters");
            let props = &mut params.properties();

            let prop = assert_next_prop(props);
            assert_eq!(prop.name().as_ident().as_ref(), "q");
            assert_prim(assert_term(prop.rhs()), lex::Primitive::Str);

            let prop = assert_next_prop(props);
            assert_eq!(prop.name().as_ident().as_ref(), "n");
            assert_prim(assert_term(prop.rhs()), lex::Primitive::Num);
        },
    );
}

#[test]
fn parse_decl_transfer_params() {
    parse(
        "let a = get, put { 'q str } : {} -> {};",
        |p: Program<()>| {
            println!("{:#?}", p);
            let xfer = Transfer::cast(assert_decl(p, "a").rhs()).expect("expected transfer");

            let mtds = &mut xfer.methods();
            assert_eq!(
                mtds.next().expect("expected a method").method(),
                lex::Method::Get
            );
            assert_eq!(
                mtds.next().expect("expected a method").method(),
                lex::Method::Put
            );

            let props = &mut xfer.params().properties().expect("expected parameters");

            let prop = assert_next_prop(props);
            assert_eq!(prop.name().as_ident().as_ref(), "q");
            assert_prim(assert_term(prop.rhs()), lex::Primitive::Str);

            // TODO: test domain and range
        },
    )
}
