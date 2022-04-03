mod compile;
mod errors;
mod eval;
mod inference;
mod scan;
mod scope;
mod transform;

#[cfg(test)]
mod compile_tests;
#[cfg(test)]
mod inference_tests;
#[cfg(test)]
mod scope_tests;

pub use crate::compile::compile;
pub use crate::errors::Result;
pub use crate::eval::eval;
