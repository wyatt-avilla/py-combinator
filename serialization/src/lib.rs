#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]

mod attr_list;
mod impl_block;
mod method;
mod self_function;
mod self_generic;

pub use impl_block::ImplBlock;
pub use method::Method;

pub const REGISTER_METHODS_ATTRIBUTE: &str = "register_methods";
pub const SELF_GENERIC_ATTRIBUTE: &str = "self_generic";
pub const SELF_FUNC_ATTRIBUTE: &str = "method_self_arg";
pub const RETURN_LITERAL_ATTRIBUTE: &str = "return_literal";
pub const STRIPS_TRAITS_ATTRIBUTE: &str = "strips_traits";
