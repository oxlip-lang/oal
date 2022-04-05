mod errors;
pub mod eval;
mod inference;
mod reduction;
mod scan;
mod scope;
mod transform;

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
pub use crate::reduction::reduce;
