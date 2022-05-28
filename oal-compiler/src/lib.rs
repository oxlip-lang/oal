extern crate core;

mod annotation;
mod compile;
mod errors;
mod expr;
mod inference;
mod locator;
mod module;
mod node;
mod reduction;
mod scan;
mod scope;
pub mod spec;
mod tag;
mod transform;
mod typecheck;

#[cfg(test)]
mod annotation_tests;
#[cfg(test)]
mod compile_tests;
#[cfg(test)]
mod inference_tests;
#[cfg(test)]
mod module_tests;
#[cfg(test)]
mod reduction_tests;
#[cfg(test)]
mod scope_tests;
#[cfg(test)]
mod spec_tests;
#[cfg(test)]
mod typecheck_tests;

pub use crate::compile::compile;
pub use crate::errors::Result;
pub use crate::locator::Locator;
pub use crate::module::load;

pub type Program = oal_syntax::ast::Program<expr::TypedExpr>;
pub type ModuleSet = module::ModuleSet<expr::TypedExpr>;
