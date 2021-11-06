extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::iterators::Pair;
use pest::{Parser, RuleType};
use std::fs;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct MyParser;

#[derive(Debug)]
struct Doc {
    stmts: Vec<Stmt>,
}

impl Doc {
    fn from_pair(p: Pair<Rule>) -> Doc {
        Doc {
            stmts: p
                .into_inner()
                .flat_map(|p| {
                    if let Rule::stmt = p.as_rule() {
                        Some(Stmt::from_pair(p))
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

#[derive(Debug)]
struct Stmt {}

impl Stmt {
    fn from_pair(p: Pair<Rule>) -> Stmt {
        Stmt {}
    }
}

fn main() {
    let unparsed_file = fs::read_to_string("src/doc.txt").expect("cannot read file");

    let p: Pair<_> = MyParser::parse(Rule::doc, &unparsed_file)
        .expect("parsing failed")
        .next()
        .unwrap();

    let doc = Doc::from_pair(p);
    /*
       let p: Vec<_> = p
           .into_inner()
           .flat_map(|p| {
               if let Rule::stmt = p.as_rule() {
                   let stmt = p.into_inner().next().unwrap();
                   match stmt.as_rule() {
                       Rule::decl => {
                           let mut i = stmt.into_inner();
                           let ident = i.nth(1).unwrap();
                           Some(ident.as_str())
                       }
                       Rule::res => Some("a resource"),
                       _ => None,
                   }
               } else {
                   None
               }
           })
           .collect();
    */
    println!("{:#?}", doc)
}
