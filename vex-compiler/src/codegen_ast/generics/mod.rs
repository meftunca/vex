// generics/mod.rs
// Generic type instantiation and inference

mod functions;
mod inference;
mod structs;

// Re-export public APIs
pub(crate) use functions::*;
pub(crate) use structs::*;
