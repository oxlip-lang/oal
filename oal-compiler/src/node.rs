//! Enumerations to wrap nodes when walking through an abstract syntax tree.

use oal_syntax::ast::{Annotation, Declaration, Import, Resource};

/// Immutable AST node wrapper
#[derive(Debug)]
pub enum NodeRef<'a, T> {
    Expr(&'a T),
    Decl(&'a Declaration<T>),
    Res(&'a Resource<T>),
    Ann(&'a Annotation),
    Use(&'a Import),
}

/// Mutable AST node wrapper
#[derive(Debug)]
pub enum NodeMut<'a, T> {
    Expr(&'a mut T),
    Decl(&'a mut Declaration<T>),
    Res(&'a mut Resource<T>),
    Ann(&'a mut Annotation),
    Use(&'a mut Import),
}
