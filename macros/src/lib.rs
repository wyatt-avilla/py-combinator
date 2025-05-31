#![warn(clippy::pedantic)]

use std::collections::{BTreeMap, BTreeSet};

use proc_macro::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemImpl, parse_macro_input};

use serialization::{
    AttributeArg, AttributeArgsList, ImplBlock, PY_BASE_ITERATOR, PY_DOUBLE_ENDED_ITERATOR,
    PY_EXACT_SIZE_ITERATOR, SELF_GENERIC_ATTRIBUTE,
};

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

fn validate_selected_traits(attr: &TokenStream) -> Result<BTreeSet<String>, String> {
    let allowed_traits = BTreeSet::from([
        String::from(PY_BASE_ITERATOR),
        String::from(PY_DOUBLE_ENDED_ITERATOR),
        String::from(PY_EXACT_SIZE_ITERATOR),
    ]);

    let selected_traits: BTreeSet<_> = syn::parse2::<AttributeArgsList>(attr.clone().into())
        .map_err(|e| e.to_string())?
        .0
        .into_iter()
        .filter_map(|aa| match aa {
            AttributeArg::Arg(a) => Some(a),
            AttributeArg::Group(g) => match g.content.0.first() {
                Some(AttributeArg::Arg(a)) => Some(a.clone()),
                _ => None,
            },
            AttributeArg::KeyValueArg(_) => None,
        })
        .map(|a| a.to_string())
        .collect();

    if selected_traits.is_empty() {
        return Err("Empty trait list".to_string());
    }

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

    match validate_selected_traits(&attr) {
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

#[proc_macro_attribute]
pub fn add_trait_methods(attr: TokenStream, token_stream: TokenStream) -> TokenStream {
    let added_traits = match validate_selected_traits(&attr) {
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

    for trait_name in &added_traits {
        let impl_block = trait_to_impl_block.get(trait_name).unwrap();
        for method in &impl_block.methods {
            if method.name == impl_block.self_function {
                continue;
            }

            let impl_item = match method.into_impl_item(impl_block) {
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
