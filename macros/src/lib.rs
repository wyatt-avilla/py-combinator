#![warn(clippy::pedantic)]

use std::collections::BTreeSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemImpl, parse_macro_input};

use serialization::SELF_GENERIC_ATTRIBUTE;

#[proc_macro_attribute]
pub fn register_methods(attr: TokenStream, token_stream: TokenStream) -> TokenStream {
    let unchanged = token_stream.clone();

    if parse_macro_input!(attr as syn::MetaNameValue)
        .path
        .get_ident()
        .is_none_or(|k| *k.to_string() != *SELF_GENERIC_ATTRIBUTE)
    {
        let e = format!(
            "expected an assignment to `self_generic` (e.g #[register_methods({SELF_GENERIC_ATTRIBUTE} = S)])"
        );
        return quote! {
            compile_error!(#e);
        }
        .into();
    }

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

#[proc_macro_attribute]
pub fn strips_traits(attr: TokenStream, token_stream: TokenStream) -> TokenStream {
    let unchanged = token_stream.clone();

    let allowed_traits = BTreeSet::from([
        String::from("PyBaseIterator"),
        String::from("PyDoubleEndedIterator"),
        String::from("PyExactSizeIterator"),
    ]);

    let stripped_traits = attr
        .into_iter()
        .filter_map(|tt| {
            if let proc_macro::TokenTree::Ident(i) = tt {
                Some(i.to_string())
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();

    if !stripped_traits.is_subset(&allowed_traits) {
        let e = format!("Invalid trait to strip, expected one of {allowed_traits:#?}",);
        return quote! {
            compile_error!(#e);
        }
        .into();
    }

    unchanged
}

#[proc_macro_attribute]
pub fn return_literal(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
    token_stream
}

#[proc_macro_attribute]
pub fn method_self_arg(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
    token_stream
}
