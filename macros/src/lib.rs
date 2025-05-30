#![warn(clippy::pedantic)]

use std::collections::BTreeSet;

use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, ImplItem, ImplItemFn, ItemImpl, parse_macro_input, parse_str};

use serialization::{ImplBlock, SELF_GENERIC_ATTRIBUTE, SERIALIZED_METHODS_PATH};

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

#[proc_macro_attribute]
#[allow(clippy::too_many_lines)]
pub fn add_trait_methods(attr: TokenStream, token_stream: TokenStream) -> TokenStream {
    let allowed_traits = BTreeSet::from([
        String::from("PyBaseIterator"),
        String::from("PyDoubleEndedIterator"),
        String::from("PyExactSizeIterator"),
    ]);

    let added_traits = attr
        .into_iter()
        .filter_map(|tt| {
            if let proc_macro::TokenTree::Ident(i) = tt {
                Some(i.to_string())
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();

    if !added_traits.is_subset(&allowed_traits) {
        let e = format!("Invalid trait to add, expected one of {allowed_traits:#?}",);
        return quote! {
            compile_error!(#e);
        }
        .into();
    }

    let mut input = parse_macro_input!(token_stream as ItemImpl);

    let file = match std::fs::File::open("py-combinator/target/iterator_methods.json") {
        Ok(f) => f,
        Err(fs_e) => {
            let e = format!("Couldn't open serialized methods file ({fs_e})",);
            return quote! {
                compile_error!(#e);
            }
            .into();
        }
    };

    let deserialized: Vec<ImplBlock> = match serde_json::from_reader(file) {
        Ok(d) => d,
        Err(ser_e) => {
            let e = format!("Couldn't deserialize from methods file ({ser_e})",);
            return quote! {
                compile_error!(#e);
            }
            .into();
        }
    };

    for impl_block in deserialized {
        let qualified_trait_name = parse_str::<syn::Path>(impl_block.nice_name().as_ref()).unwrap();
        for method in impl_block.methods {
            let method_name = parse_str::<Ident>(&method.name).unwrap();
            let arg_names: Vec<Ident> = method
                .args
                .iter()
                .filter_map(|a| {
                    if a.expected_type == impl_block.self_generic {
                        None
                    } else {
                        Some(parse_str::<Ident>(&a.name).unwrap())
                    }
                })
                .collect();

            let typed_args: TokenStream2 = Itertools::intersperse(
                method
                    .args
                    .iter()
                    .filter_map(|arg| {
                        if arg.expected_type == impl_block.self_generic {
                            None
                        } else {
                            let name = parse_str::<Ident>(&arg.name).unwrap();
                            let ty = parse_str::<syn::Type>(&arg.expected_type).unwrap();
                            Some(quote! { #name: #ty })
                        }
                    })
                    .collect::<Vec<_>>()
                    .into_iter(),
                quote! { , },
            )
            .collect();

            if method.literal_return {
                let return_type: Option<syn::Type> = method
                    .return_type
                    .as_ref()
                    .and_then(|ret| parse_str::<syn::Type>(ret).ok());

                let return_tokens = if let Some(ret_ty) = return_type {
                    quote! { -> #ret_ty }
                } else {
                    quote! {}
                };

                let call_args: TokenStream2 = Itertools::intersperse(
                    arg_names.iter().map(|name| quote! { #name }),
                    quote! { , },
                )
                .collect();

                let self_function_str = impl_block.self_function.clone();
                let self_function: TokenStream2 = parse_str(&self_function_str).unwrap();

                let test_quote = quote! {
                    pub fn #method_name(&mut self , #typed_args) #return_tokens {
                        #qualified_trait_name :: #method_name(self.#self_function(), #call_args)
                    }
                };
                dbg!(test_quote.to_string());

                dbg!(&method_name);
                let impl_item_fn: ImplItemFn = syn::parse_quote! {
                    pub fn #method_name(&mut self , #typed_args) #return_tokens {
                        #qualified_trait_name :: #method_name (self.#self_function() , #call_args)
                    }
                };

                input.items.push(ImplItem::Fn(impl_item_fn));
            } else {
                todo!()
            }
        }
    }

    quote!(#input).into()
}
