use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use itertools::Itertools;
use quote::ToTokens;
use serde::{Deserialize, Serialize};
use syn::{Ident, ImplItemFn, ItemImpl, parse_str};

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{
    ImplBlock, RETURN_LITERAL_ATTRIBUTE, STRIPS_TRAITS_ATTRIBUTE,
    attr_list::{AttributeArg, AttributeArgsList},
    impl_block::ImplBlockParseError,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Argument {
    pub mutable: bool,
    pub name: String,
    pub expected_type: String,
}

impl Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.mutable {
            write!(f, "mut {}: {}", self.name, self.expected_type)
        } else {
            write!(f, "{}: {}", self.name, self.expected_type)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Method {
    pub comments: Option<String>,
    pub name: String,
    pub args: Vec<Argument>,
    pub return_type: Option<String>,
    pub literal_return: bool,
    pub strips: Vec<String>,
}

impl Method {
    pub fn from(
        impl_block: &ItemImpl,
        fn_context: &ImplItemFn,
    ) -> Result<Method, ImplBlockParseError> {
        let name = fn_context.sig.ident.to_string();
        let args = fn_context
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(arg) => Some(arg),
            })
            .map(|arg| {
                if let syn::Pat::Ident(i) = *arg.clone().pat {
                    Ok(Argument {
                        mutable: i.mutability.is_some(),
                        name: i.ident.to_string(),
                        expected_type: arg.ty.to_token_stream().to_string(),
                    })
                } else {
                    Err(ImplBlockParseError::PatDestructure)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let literal_returns: BTreeSet<_> =
            ImplBlock::find_method_with_attribute_containing(impl_block, RETURN_LITERAL_ATTRIBUTE)
                .into_iter()
                .map(|(func, _)| func.sig.ident.to_string())
                .collect();

        let strips_map: BTreeMap<_, _> =
            ImplBlock::find_method_with_attribute_containing(impl_block, STRIPS_TRAITS_ATTRIBUTE)
                .into_iter()
                .map(|(func, attr)| -> Result<(_, _), ImplBlockParseError> {
                    Ok((
                        func.sig.ident.to_string(),
                        attr.parse_args::<AttributeArgsList>()
                            .map(|list| {
                                list.0
                                    .clone()
                                    .into_iter()
                                    .filter_map(|arg| match arg {
                                        AttributeArg::Arg(arg) => Some(arg.to_string()),
                                        AttributeArg::KeyValueArg(_) => None,
                                    })
                                    .collect_vec()
                            })
                            .map_err(|e| ImplBlockParseError::AttributeParseError(e.to_string()))?,
                    ))
                })
                .collect::<Result<_, _>>()?;

        let return_type = match &fn_context.sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, t) => Some(t.into_token_stream().to_string()),
        };

        let literal_return = literal_returns.iter().contains(&name);

        let strips = strips_map.get(&name).unwrap_or(&vec![]).clone();

        Ok(Method {
            comments: None,
            name,
            args,
            return_type,
            literal_return,
            strips,
        })
    }

    pub fn vec_from(impl_block: &ItemImpl) -> Result<Vec<Method>, ImplBlockParseError> {
        impl_block
            .items
            .iter()
            .filter_map(|i| {
                if let syn::ImplItem::Fn(fn_context) = i {
                    Some(fn_context)
                } else {
                    None
                }
            })
            .map(|fn_context| Method::from(impl_block, fn_context))
            .collect()
    }

    pub fn into_impl_item(&self, impl_block: &ImplBlock) -> Result<ImplItemFn, String> {
        let qualified_trait_name =
            parse_str::<syn::Path>(impl_block.nice_name().as_ref()).map_err(|e| e.to_string())?;

        let self_name = parse_str::<Ident>(&self.name).map_err(|e| e.to_string())?;
        let arg_names: Vec<Ident> = self
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
            self.args
                .iter()
                .filter(|arg| arg.expected_type != impl_block.self_generic)
                .map(|arg| {
                    let name = parse_str::<Ident>(&arg.name).map_err(|e| e.to_string())?;
                    let ty =
                        parse_str::<syn::Type>(&arg.expected_type).map_err(|e| e.to_string())?;
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
            let return_type = if self.literal_return {
                self.return_type
                    .as_ref()
                    .map(|ret| parse_str::<syn::Type>(ret).map_err(|e| e.to_string()))
                    .transpose()?
            } else {
                if self.strips.is_empty() {
                    Some(parse_str::<syn::Type>("Self").map_err(|e| e.to_string())?)
                } else {
                    let impl_name = impl_block
                        .name
                        .last()
                        .ok_or("Empty impl block name".to_string())?;
                    todo!()
                }
            };

            if let Some(ret_ty) = return_type {
                quote! { -> #ret_ty }
            } else {
                quote! {}
            }
        };

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
