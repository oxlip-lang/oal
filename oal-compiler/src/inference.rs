use oal_syntax::ast::{Decl, Doc, Expr, Res, Stmt, Tag};

pub struct TypeEquation {
    pub left: Tag,
    pub right: Tag,
}

pub trait TypedNode {
    fn equations(&self, eqs: &mut Vec<TypeEquation>);
}

impl TypedNode for Expr {
    fn equations(&self, eqs: &mut Vec<TypeEquation>) {
        match self {
            Expr::Prim(_) => {}
            Expr::Rel(_) => {}
            Expr::Uri(_) => {}
            Expr::Join(_) => {}
            Expr::Block(_) => {}
            Expr::Sum(_) => {}
            Expr::Var(_) => {}
        }
    }
}

impl TypedNode for Decl {
    fn equations(&self, eqs: &mut Vec<TypeEquation>) {
        self.body.expr.equations(eqs);
        eqs.push(TypeEquation {
            left: todo!(),
            right: todo!(),
        })
    }
}

impl TypedNode for Res {
    fn equations(&self, eqs: &mut Vec<TypeEquation>) {
        self.rel.expr.equations(eqs)
    }
}

impl TypedNode for Stmt {
    fn equations(&self, eqs: &mut Vec<TypeEquation>) {
        match self {
            Stmt::Decl(d) => d.equations(eqs),
            Stmt::Res(r) => r.equations(eqs),
        }
    }
}

trait Tagged {
    fn tag_type(&mut self, n: usize) -> usize;
}

impl Tagged for Decl {
    fn tag_type(&mut self, n: usize) -> usize {
        todo!()
    }
}

impl Tagged for Res {
    fn tag_type(&mut self, n: usize) -> usize {
        todo!()
    }
}

impl Tagged for Doc {
    fn tag_type(&mut self, n: usize) -> usize {
        let mut n = n;
        for s in self.stmts.iter_mut() {
            n = match s {
                Stmt::Decl(d) => d.tag_type(n),
                Stmt::Res(r) => r.tag_type(n),
            }
        }
        n
    }
}
