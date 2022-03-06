mod compile;
mod errors;
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
pub use crate::inference::{constrain, substitute, tag_type, TagSeq, TypeConstraint};
pub use crate::scan::Scan;
pub use crate::scope::Env;
pub use crate::transform::Transform;
