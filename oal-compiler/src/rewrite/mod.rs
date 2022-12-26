pub mod compile;
pub mod env;
pub mod eval;
pub mod infer;
pub mod module;
pub mod resolve;
pub mod tree;
pub mod typecheck;

#[cfg(test)]
mod compile_tests;
#[cfg(test)]
mod eval_tests;
#[cfg(test)]
mod infer_tests;
#[cfg(test)]
mod resolve_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod typecheck_tests;
