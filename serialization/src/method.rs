use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use itertools::Itertools;
use quote::ToTokens;
use serde::{Deserialize, Serialize};
use syn::{ImplItemFn, ItemImpl};

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
}
