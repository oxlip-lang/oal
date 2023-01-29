use crate::definition::{Definition, Internal};
use crate::env::Env;
use crate::errors::Result;
use crate::eval::{cast_uri, eval_terminal, AnnRef, Context, Expr, Value};
use crate::inference::tag;
use crate::tree::Core;
use oal_syntax::parser as syn;
use std::rc::Rc;

#[derive(Debug)]
struct Concat {}

impl Internal for Concat {
    fn tag(&self, _seq: &mut tag::Seq) -> tag::Tag {
        let f = tag::FuncTag {
            bindings: vec![tag::Tag::Uri, tag::Tag::Uri],
            range: Box::new(tag::Tag::Uri),
        };
        tag::Tag::Func(f)
    }

    fn eval<'a>(
        &self,
        ctx: &mut Context<'a>,
        mut args: Vec<syn::Terminal<'a, Core>>,
        ann: AnnRef,
    ) -> Result<Value<'a>> {
        assert_eq!(args.len(), 2);
        let right = cast_uri(eval_terminal(ctx, args.pop().unwrap(), AnnRef::default())?);
        let mut left = cast_uri(eval_terminal(ctx, args.pop().unwrap(), AnnRef::default())?);
        left.append(right);
        let expr = Expr::Uri(Box::new(left));
        Ok((expr, ann))
    }

    fn has_bindings(&self) -> bool {
        true
    }
}

pub fn import(env: &mut Env) {
    let internals = [("concat", Rc::new(Concat {}))];
    for i in internals.into_iter() {
        env.declare(i.0.into(), Definition::Internal(i.1))
    }
}
