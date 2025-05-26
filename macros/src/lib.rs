#![warn(clippy::pedantic)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemImpl, parse_macro_input};

#[proc_macro_attribute]
pub fn register_methods(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
    let unchanged = token_stream.clone();

    if let syn::Type::Path(p) = *parse_macro_input!(token_stream as ItemImpl).self_ty {
        let path_segments: Vec<_> = p
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();

        if !path_segments.starts_with(&["crate".to_string()])
            || path_segments.iter().any(|s| s == "super" || s == "self")
        {
            return quote! {
                compile_error!("usage of this macro requires a fully qualified path starting with `crate::`, and not containing `self` or `super`");
            }
            .into();
        }
    } else {
        return quote! {
            compile_error!("expected a path type in impl block (e.g., impl crate::foo::Bar), got something else");
        }
        .into();
    }

    unchanged
}
