extern crate core;

mod annotation;
mod errors;
pub mod eval;
mod expr;
mod inference;
mod reduction;
mod scan;
mod scope;
mod tag;
mod transform;

#[cfg(test)]
mod annotation_tests;
#[cfg(test)]
mod eval_tests;
#[cfg(test)]
mod inference_tests;
#[cfg(test)]
mod reduction_tests;
#[cfg(test)]
mod scope_tests;

pub use crate::errors::Result;
pub use crate::eval::evaluate;

pub type Program = oal_syntax::ast::Program<expr::TypedExpr>;
