mod annotation;
pub mod compile;
mod env;
mod errors;
pub mod eval;
mod inference;
pub mod module;
mod resolve;
pub mod spec;
pub mod tree;
mod typecheck;

#[cfg(test)]
mod compile_tests;
#[cfg(test)]
mod eval_tests;
#[cfg(test)]
mod module_tests;
#[cfg(test)]
mod resolve_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod typecheck_tests;
