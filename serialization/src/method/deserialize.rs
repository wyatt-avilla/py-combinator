use std::collections::HashMap;

use itertools::Itertools;
use syn::{Ident, ImplItemFn, parse_str};

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use thiserror::Error;

use crate::{
    ImplBlock, Method, PY_BASE_ITERATOR, PY_DOUBLE_ENDED_ITERATOR, PY_EXACT_SIZE_ITERATOR,
    PY_SIZED_DOUBLE_ENDED_ITERATOR, method::Argument,
};

#[derive(Debug, Error)]
pub enum MethodDeserializeError {
    #[error("Couldn't parse Method name into a `syn::Path`")]
    NamePathParseError(String),

    #[error("Couldn't parse name into a `syn::Ident`")]
    NameParseError(String),

    #[error("Method field was empty")]
    EmptyField,

    #[error("Couldn't parse type into a `syn::Type`")]
    ArgTypeParseError(String),

    #[error("Couldn't parse into `TokenStream`")]
    TokenStreamParseError(String),

    #[error("Invalid iterator name")]
    InvalidIteratorName,
}

fn arg_names_from(
    args: &[Argument],
    impl_block: &ImplBlock,
) -> Result<Vec<Ident>, MethodDeserializeError> {
    args.iter()
        .filter_map(|a| {
            if a.expected_type == impl_block.self_generic {
                None
            } else {
                Some(parse_str::<Ident>(&a.name))
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| MethodDeserializeError::NameParseError(e.to_string()))
}

fn typed_args_from(
    args: &[Argument],
    impl_block: &ImplBlock,
) -> Result<TokenStream2, MethodDeserializeError> {
    let typed_args = args
        .iter()
        .filter(|arg| arg.expected_type != impl_block.self_generic)
        .map(|arg| {
            let name = parse_str::<Ident>(&arg.name)
                .map_err(|e| MethodDeserializeError::NameParseError(e.to_string()))?;
            let ty = parse_str::<syn::Type>(&arg.expected_type)
                .map_err(|e| MethodDeserializeError::ArgTypeParseError(e.to_string()))?;
            Ok(quote! { #name: #ty })
        })
        .collect::<Result<Vec<_>, _>>()?;

    if args.len() == 1 {
        Ok(typed_args.into_iter().collect())
    } else {
        Ok([quote! { , }]
            .into_iter()
            .chain(Itertools::intersperse(typed_args.into_iter(), quote! { , }))
            .collect())
    }
}

fn call_args_from(arg_names: &[Ident]) -> TokenStream2 {
    if arg_names.is_empty() {
        quote! {}
    } else {
        [quote! { , }]
            .into_iter()
            .chain(Itertools::intersperse(
                arg_names.iter().map(|name| quote! { #name }),
                quote! { , },
            ))
            .collect()
    }
}

fn return_tokens_from(
    method: &Method,
    impl_block: &ImplBlock,
) -> Result<TokenStream2, MethodDeserializeError> {
    let return_type = if method.literal_return {
        method
            .return_type
            .as_ref()
            .map(|ret| {
                parse_str::<syn::Type>(ret)
                    .map_err(|e| MethodDeserializeError::ArgTypeParseError(e.to_string()))
            })
            .transpose()?
    } else if method.strips.is_empty() {
        Some(
            parse_str::<syn::Type>("Self")
                .map_err(|e| MethodDeserializeError::ArgTypeParseError(e.to_string()))?,
        )
    } else {
        let impl_name = impl_block
            .name
            .last()
            .ok_or(MethodDeserializeError::EmptyField)?;

        let trait_map: HashMap<_, Vec<_>> = [
            (PY_BASE_ITERATOR, vec![]),
            (
                PY_DOUBLE_ENDED_ITERATOR,
                vec![PY_BASE_ITERATOR, PY_DOUBLE_ENDED_ITERATOR],
            ),
            (
                PY_EXACT_SIZE_ITERATOR,
                vec![PY_BASE_ITERATOR, PY_EXACT_SIZE_ITERATOR],
            ),
            (
                PY_SIZED_DOUBLE_ENDED_ITERATOR,
                vec![
                    PY_BASE_ITERATOR,
                    PY_EXACT_SIZE_ITERATOR,
                    PY_DOUBLE_ENDED_ITERATOR,
                ],
            ),
        ]
        .into_iter()
        .collect();

        let available_traits = trait_map
            .get(impl_name.as_str())
            .ok_or(MethodDeserializeError::InvalidIteratorName)?;

        let remaining_traits = available_traits
            .iter()
            .filter(|&&t| !method.strips.contains(&t.to_string()))
            .copied()
            .collect_vec();

        Some(
            parse_str::<syn::Type>(
                &remaining_traits
                    .last()
                    .map_or(format!("crate::iterators::{PY_BASE_ITERATOR}"), |rt| {
                        format!("crate::iterators:: {rt}")
                    }),
            )
            .map_err(|e| MethodDeserializeError::ArgTypeParseError(e.to_string()))?,
        )
    };

    if let Some(ret_ty) = return_type {
        Ok(quote! { #ret_ty })
    } else {
        Ok(quote! {})
    }
}

impl Method {
    pub fn into_impl_item(
        &self,
        impl_block: &ImplBlock,
    ) -> Result<ImplItemFn, MethodDeserializeError> {
        let qualified_trait_name = parse_str::<syn::Path>(impl_block.nice_name().as_ref())
            .map_err(|e| MethodDeserializeError::NameParseError(e.to_string()))?;

        let self_name = parse_str::<Ident>(&self.name)
            .map_err(|e| MethodDeserializeError::NamePathParseError(e.to_string()))?;

        let arg_names = arg_names_from(&self.args, impl_block)?;
        let typed_args = typed_args_from(&self.args, impl_block)?;
        let call_args = call_args_from(&arg_names);

        let self_function: TokenStream2 = parse_str(&impl_block.self_function.clone())
            .map_err(|e| MethodDeserializeError::TokenStreamParseError(e.to_string()))?;

        let return_type = return_tokens_from(self, impl_block)?;

        let test_quote = if self.literal_return {
            quote! {
                pub fn #self_name(&mut self #typed_args) -> #return_type {
                    #qualified_trait_name :: #self_name (self.#self_function() #call_args)
                }
            }
        } else if return_type.is_empty() {
            quote! {
                pub fn #self_name(&mut self #typed_args) {
                    ::std::boxed::Box::new ( #qualified_trait_name :: #self_name (self.#self_function() #call_args) )
                }
            }
        } else {
            quote! {
                pub fn #self_name(&mut self #typed_args) -> #return_type {
                    #return_type ::new( ::std::boxed::Box::new ( #qualified_trait_name :: #self_name (self.#self_function() #call_args) ) )
                }
            }
        };
        dbg!(test_quote.to_string());

        let impl_item_fn: ImplItemFn = if self.literal_return {
            syn::parse_quote! {
                pub fn #self_name(&mut self #typed_args) -> #return_type {
                    #qualified_trait_name :: #self_name (self.#self_function() #call_args)
                }
            }
        } else if return_type.is_empty() {
            syn::parse_quote! {
                pub fn #self_name(&mut self #typed_args) {
                    ::std::boxed::Box::new ( #qualified_trait_name :: #self_name (self.#self_function() #call_args) )
                }
            }
        } else {
            syn::parse_quote! {
                pub fn #self_name(&mut self #typed_args) -> #return_type {
                    #return_type ::new( ::std::boxed::Box::new ( #qualified_trait_name :: #self_name (self.#self_function() #call_args) ) )
                }
            }
        };

        Ok(impl_item_fn)
    }
}
