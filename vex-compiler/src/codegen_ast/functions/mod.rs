// src/codegen/functions/mod.rs
pub(crate) mod declare;
pub(crate) mod compile;
pub(crate) mod asynchronous;

pub(crate) use declare::*;
pub(crate) use compile::*;
pub(crate) use asynchronous::*;
