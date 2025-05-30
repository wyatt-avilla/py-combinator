use itertools::Itertools;
use syn::{Ident, ImplItemFn, parse_str};

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{ImplBlock, Method};

impl Method {
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
