#![warn(clippy::pedantic)]

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn register_methods(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
