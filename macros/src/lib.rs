#![warn(clippy::pedantic)]

use std::collections::{BTreeMap, BTreeSet};

use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, ImplItem, ImplItemFn, ItemImpl, parse_macro_input, parse_str};

use serialization::{ImplBlock, Method, SELF_GENERIC_ATTRIBUTE};

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

fn validate_selected_traits(attr: TokenStream) -> Result<BTreeSet<String>, String> {
    let allowed_traits = BTreeSet::from([
        String::from("PyBaseIterator"),
        String::from("PyDoubleEndedIterator"),
        String::from("PyExactSizeIterator"),
    ]);

    let selected_traits = attr
        .into_iter()
        .filter_map(|tt| {
            if let proc_macro::TokenTree::Ident(i) = tt {
                Some(i.to_string())
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();

    if selected_traits.is_subset(&allowed_traits) {
        Ok(selected_traits)
    } else {
        Err(format!(
            "Invalid trait to strip, expected one of {allowed_traits:#?}",
        ))
    }
}

#[proc_macro_attribute]
pub fn strips_traits(attr: TokenStream, token_stream: TokenStream) -> TokenStream {
    let unchanged = token_stream.clone();

    match validate_selected_traits(attr) {
        Ok(_) => {}
        Err(e) => {
            return quote! {
                compile_error!(#e);
            }
            .into();
        }
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

fn method_into_impl_item(method: &Method, impl_block: &ImplBlock) -> Result<ImplItemFn, String> {
    let qualified_trait_name =
        parse_str::<syn::Path>(impl_block.nice_name().as_ref()).map_err(|e| e.to_string())?;

    let method_name = parse_str::<Ident>(&method.name).map_err(|e| e.to_string())?;
    let arg_names: Vec<Ident> = method
        .args
        .iter()
        .filter_map(|a| {
            if a.expected_type == impl_block.self_generic {
                None
            } else {
                Some(parse_str::<Ident>(&a.name))
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let typed_args: TokenStream2 = Itertools::intersperse(
        method
            .args
            .iter()
            .filter(|arg| arg.expected_type != impl_block.self_generic)
            .map(|arg| {
                let name = parse_str::<Ident>(&arg.name).map_err(|e| e.to_string())?;
                let ty = parse_str::<syn::Type>(&arg.expected_type).map_err(|e| e.to_string())?;
                Ok(quote! { #name: #ty })
            })
            .collect::<Result<Vec<_>, String>>()?
            .into_iter(),
        quote! { , },
    )
    .collect();

    let call_args: TokenStream2 =
        Itertools::intersperse(arg_names.iter().map(|name| quote! { #name }), quote! { , })
            .collect();

    let self_function: TokenStream2 =
        parse_str(&impl_block.self_function.clone()).map_err(|e| e.to_string())?;

    let return_tokens: TokenStream2 = {
        let return_type = method
            .return_type
            .as_ref()
            .map(|ret| parse_str::<syn::Type>(ret).map_err(|e| e.to_string()))
            .transpose()?;

        if let Some(ret_ty) = return_type {
            quote! { -> #ret_ty }
        } else {
            quote! {}
        }
    };

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

    Ok(impl_item_fn)
}

#[proc_macro_attribute]
pub fn add_trait_methods(attr: TokenStream, token_stream: TokenStream) -> TokenStream {
    let added_traits = match validate_selected_traits(attr) {
        Ok(t) => t,
        Err(e) => {
            return quote! {
                compile_error!(#e);
            }
            .into();
        }
    };

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

    let trait_to_impl_block = match match serde_json::from_reader::<_, Vec<ImplBlock>>(file) {
        Ok(d) => d,
        Err(ser_e) => {
            let e = format!("Couldn't deserialize from methods file ({ser_e})",);
            return quote! {
                compile_error!(#e);
            }
            .into();
        }
    }
    .into_iter()
    .map(|ib| match ib.name.last() {
        Some(name) => Ok((name.clone(), ib)),
        None => Err("Impl block with empty name".to_string()),
    })
    .collect::<Result<BTreeMap<_, _>, _>>()
    {
        Ok(map) => map,
        Err(e) => {
            return quote! {
                compile_error!(#e);
            }
            .into();
        }
    };

    let mut input = parse_macro_input!(token_stream as ItemImpl);
    dbg!(&added_traits);

    for trait_name in &added_traits {
        let impl_block = trait_to_impl_block.get(trait_name).unwrap();
        for method in &impl_block.methods {
            if method.name == impl_block.self_function {
                continue;
            }

            let impl_item = match method_into_impl_item(method, impl_block) {
                Ok(ii) => ii,
                Err(e) => {
                    let e = format!("Couldn't parse method ({e})",);
                    return quote! {
                        compile_error!(#e);
                    }
                    .into();
                }
            };

            input.items.push(ImplItem::Fn(impl_item));
        }
    }

    quote!(#input).into()
}
