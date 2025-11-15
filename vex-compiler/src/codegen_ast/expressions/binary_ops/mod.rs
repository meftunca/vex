// Binary operations - split into logical modules for maintainability

pub(crate) mod enum_ops;
pub(crate) mod float_ops;
pub(crate) mod integer_ops;
pub(crate) mod operator_overloading;
pub(crate) mod pointer_loading;
pub(crate) mod pointer_ops;
pub(crate) mod power_ops;
pub(crate) mod struct_ops;
pub(crate) mod type_alignment;

pub(crate) use enum_ops::*;
pub(crate) use float_ops::*;
pub(crate) use integer_ops::*;
pub(crate) use operator_overloading::*;
pub(crate) use pointer_loading::*;
pub(crate) use pointer_ops::*;
pub(crate) use power_ops::*;
pub(crate) use struct_ops::*;
pub(crate) use type_alignment::*;
