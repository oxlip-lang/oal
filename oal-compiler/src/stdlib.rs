use crate::definition::{Definition, Internal};
use crate::env::Env;
use crate::errors::Result;
use crate::eval::{cast_uri, AnnRef, Expr, Value};
use crate::inference::tag;
use std::rc::Rc;

#[repr(u32)]
enum Identifier {
    Concat
}

#[derive(Debug)]
pub struct Concat;

impl Internal for Concat {
    fn tag(&self, _seq: &mut tag::Seq) -> tag::Tag {
        let f = tag::FuncTag {
            bindings: vec![tag::Tag::Uri, tag::Tag::Uri],
            range: Box::new(tag::Tag::Uri),
        };
        tag::Tag::Func(f)
    }

    fn eval<'a>(&self, mut args: Vec<Value<'a>>, ann: AnnRef) -> Result<Value<'a>> {
        assert_eq!(args.len(), 2);
        let right = cast_uri(args.pop().unwrap());
        let mut left = cast_uri(args.pop().unwrap());
        left.append(right);
        let expr = Expr::Uri(Box::new(left));
        Ok((expr, ann))
    }

    fn has_bindings(&self) -> bool {
        true
    }

    fn id(&self) -> u32 {
        Identifier::Concat as u32
    }
}

/// Imports the standard library into the given environment.
pub fn import(env: &mut Env) {
    let internals = [("concat", Rc::new(Concat {}))];
    for i in internals.into_iter() {
        env.declare(i.0.into(), Definition::Internal(i.1))
    }
}
