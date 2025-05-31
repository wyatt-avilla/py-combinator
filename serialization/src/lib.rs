#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]

mod attr_list;
mod impl_block;
mod method;
mod self_function;
mod self_generic;

pub use attr_list::{AttributeArg, AttributeArgsList};
pub use impl_block::{ImplBlock, ImplBlockParseError};
pub use method::Method;

pub const REGISTER_METHODS_ATTRIBUTE: &str = "register_methods";
pub const SELF_GENERIC_ATTRIBUTE: &str = "self_generic";
pub const SELF_FUNC_ATTRIBUTE: &str = "method_self_arg";
pub const RETURN_LITERAL_ATTRIBUTE: &str = "return_literal";
pub const STRIPS_TRAITS_ATTRIBUTE: &str = "strips_traits";
pub const SERIALIZED_METHODS_PATH: &str = "target/iterator_methods.json";

pub const PY_BASE_ITERATOR: &str = "PyBaseIterator";
pub const PY_DOUBLE_ENDED_ITERATOR: &str = "PyDoubleEndedIterator";
pub const PY_EXACT_SIZE_ITERATOR: &str = "PyExactSizeIterator";
pub const PY_SIZED_DOUBLE_ENDED_ITERATOR: &str = "PySizedDoubleEndedIterator";
