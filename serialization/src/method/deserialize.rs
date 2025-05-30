use itertools::Itertools;
use syn::{Ident, ImplItemFn, parse_str};

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use thiserror::Error;

use crate::{ImplBlock, Method, method::Argument};

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
    Ok(Itertools::intersperse(
        args.iter()
            .filter(|arg| arg.expected_type != impl_block.self_generic)
            .map(|arg| {
                let name = parse_str::<Ident>(&arg.name)
                    .map_err(|e| MethodDeserializeError::NameParseError(e.to_string()))?;
                let ty = parse_str::<syn::Type>(&arg.expected_type)
                    .map_err(|e| MethodDeserializeError::ArgTypeParseError(e.to_string()))?;
                Ok(quote! { #name: #ty })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter(),
        quote! { , },
    )
    .collect())
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
    } else {
        if method.strips.is_empty() {
            Some(
                parse_str::<syn::Type>("Self")
                    .map_err(|e| MethodDeserializeError::ArgTypeParseError(e.to_string()))?,
            )
        } else {
            let impl_name = impl_block
                .name
                .last()
                .ok_or(MethodDeserializeError::EmptyField)?;
            todo!()
        }
    };

    if let Some(ret_ty) = return_type {
        Ok(quote! { -> #ret_ty })
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
        let call_args: TokenStream2 =
            Itertools::intersperse(arg_names.iter().map(|name| quote! { #name }), quote! { , })
                .collect();

        let self_function: TokenStream2 = parse_str(&impl_block.self_function.clone())
            .map_err(|e| MethodDeserializeError::TokenStreamParseError(e.to_string()))?;

        let return_tokens = return_tokens_from(self, impl_block)?;

        let test_quote = quote! {
            pub fn #self_name(&mut self , #typed_args) #return_tokens {
                #qualified_trait_name :: #self_name(self.#self_function(), #call_args)
            }
        };
        dbg!(test_quote.to_string());

        dbg!(&self_name);
        let impl_item_fn: ImplItemFn = syn::parse_quote! {
            pub fn #self_name(&mut self , #typed_args) #return_tokens {
                #qualified_trait_name :: #self_name (self.#self_function() , #call_args)
            }
        };

        Ok(impl_item_fn)
    }
}
