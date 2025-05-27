#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]

mod impl_block;
mod method;

pub use impl_block::ImplBlock;
pub use method::Method;

pub const REGISTER_METHODS_ATTRIBUTE: &str = "register_methods";
pub const SELF_GENERIC_ATTRIBUTE: &str = "self_generic";
pub const SELF_FUNC_ATTRIBUTE: &str = "method_self_arg";
