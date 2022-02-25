use crate::Env;
use oal_syntax::ast::*;

pub trait Transform {
    fn transform<F, E>(&self, env: &mut Env, f: F) -> Result<Self, E>
    where
        Self: Sized,
        E: Sized,
        F: FnMut(&mut Env, &Expr) -> Result<Expr, E>;
}

impl Transform for TypedExpr {
    fn transform<F, E>(&self, env: &mut Env, mut f: F) -> Result<Self, E>
    where
        F: FnMut(&mut Env, &Expr) -> Result<Expr, E>,
    {
        f(env, &self.expr).map(|e| TypedExpr {
            tag: self.tag,
            expr: e,
        })
    }
}

impl Transform for Doc {
    fn transform<F, E>(&self, env: &mut Env, mut f: F) -> Result<Self, E>
    where
        F: FnMut(&mut Env, &Expr) -> Result<Expr, E>,
    {
        let stmts: Result<Vec<_>, _> = self
            .stmts
            .iter()
            .map(|s| s.transform(env, |v, e| f(v, e)))
            .collect();
        stmts.map(|stmts| Doc { stmts })
    }
}

impl Transform for Decl {
    fn transform<F, E>(&self, env: &mut Env, f: F) -> Result<Self, E>
    where
        F: FnMut(&mut Env, &Expr) -> Result<Expr, E>,
    {
        self.body.transform(env, f).map(|body| {
            env.declare(&self.var, &body);
            Decl {
                var: self.var.clone(),
                body,
            }
        })
    }
}

impl Transform for Res {
    fn transform<F, E>(&self, env: &mut Env, f: F) -> Result<Self, E>
    where
        F: FnMut(&mut Env, &Expr) -> Result<Expr, E>,
    {
        self.rel.transform(env, f).map(|rel| Res { rel })
    }
}

impl Transform for Stmt {
    fn transform<F, E>(&self, env: &mut Env, f: F) -> Result<Self, E>
    where
        F: FnMut(&mut Env, &Expr) -> Result<Expr, E>,
    {
        match self {
            Stmt::Decl(d) => d.transform(env, f).map(Stmt::Decl),
            Stmt::Res(r) => r.transform(env, f).map(Stmt::Res),
        }
    }
}

impl Transform for Rel {
    fn transform<F, E>(&self, env: &mut Env, mut f: F) -> Result<Self, E>
    where
        F: FnMut(&mut Env, &Expr) -> Result<Expr, E>,
    {
        let uri = self.uri.transform(env, |v, e| f(v, e))?;
        let range = self.range.transform(env, |v, e| f(v, e))?;
        Ok(Rel {
            uri: uri.into(),
            methods: self.methods.clone(),
            range: range.into(),
        })
    }
}

impl Transform for UriSegment {
    fn transform<F, E>(&self, env: &mut Env, f: F) -> Result<Self, E>
    where
        F: FnMut(&mut Env, &Expr) -> Result<Expr, E>,
    {
        match self {
            UriSegment::Literal(_) => Ok(self.clone()),
            UriSegment::Template(p) => p.transform(env, f).map(UriSegment::Template),
        }
    }
}

impl Transform for Uri {
    fn transform<F, E>(&self, env: &mut Env, mut f: F) -> Result<Self, E>
    where
        F: FnMut(&mut Env, &Expr) -> Result<Expr, E>,
    {
        let spec: Result<Vec<_>, _> = self
            .spec
            .iter()
            .map(|s| s.transform(env, |v, e| f(v, e)))
            .collect();
        spec.map(|spec| Uri { spec })
    }
}

impl Transform for Prop {
    fn transform<F, E>(&self, env: &mut Env, f: F) -> Result<Self, E>
    where
        F: FnMut(&mut Env, &Expr) -> Result<Expr, E>,
    {
        self.val.transform(env, f).map(|e| Prop {
            key: self.key.clone(),
            val: e,
        })
    }
}

impl Transform for Block {
    fn transform<F, E>(&self, env: &mut Env, mut f: F) -> Result<Self, E>
    where
        F: FnMut(&mut Env, &Expr) -> Result<Expr, E>,
    {
        let props: Result<Vec<_>, _> = self
            .props
            .iter()
            .map(|p| p.transform(env, |v, e| f(v, e)))
            .collect();
        props.map(|props| Block { props })
    }
}

impl Transform for VariadicOp {
    fn transform<F, E>(&self, env: &mut Env, mut f: F) -> Result<Self, E>
    where
        F: FnMut(&mut Env, &Expr) -> Result<Expr, E>,
    {
        let exprs: Result<Vec<_>, _> = self
            .exprs
            .iter()
            .map(|e| e.transform(env, |v, e| f(v, e)))
            .collect();
        exprs.map(|exprs| VariadicOp { op: self.op, exprs })
    }
}
