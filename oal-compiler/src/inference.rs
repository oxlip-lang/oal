use oal_syntax::ast::{Stmt, StmtDecl, StmtRes, TypeExpr, TypeTag};

pub struct TypeEquation {
    pub left: TypeTag,
    pub right: TypeTag,
}

pub trait TypedNode {
    fn equations(&self, eqs: &mut Vec<TypeEquation>);
}

impl TypedNode for TypeExpr {
    fn equations(&self, eqs: &mut Vec<TypeEquation>) {
        match self {
            TypeExpr::Prim(_) => {}
            TypeExpr::Rel(_) => {}
            TypeExpr::Uri(_) => {}
            TypeExpr::Join(_) => {}
            TypeExpr::Block(_) => {}
            TypeExpr::Sum(_) => {}
            TypeExpr::Var(_) => {}
        }
    }
}

impl TypedNode for StmtDecl {
    fn equations(&self, eqs: &mut Vec<TypeEquation>) {
        self.expr.equations(eqs);
        eqs.push(TypeEquation {
            left: self.tag,
            right: todo!(),
        })
    }
}

impl TypedNode for StmtRes {
    fn equations(&self, eqs: &mut Vec<TypeEquation>) {
        self.rel.equations(eqs)
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
