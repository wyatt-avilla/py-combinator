use quote::ToTokens;
use serde::Serialize;
use syn::ItemImpl;
use thiserror::Error;

#[derive(Serialize, Debug)]
pub struct Argument {
    mutable: bool,
    name: String,
    expected_type: String,
}

#[derive(Serialize, Debug)]
pub struct Method {
    comments: Option<String>,
    name: String,
    args: Vec<Argument>,
    return_type: Option<String>,
}

#[derive(Debug, Error)]
pub enum MethodParseError {
    #[error("Couldn't destructure `PatType` into `Pat::Ident`")]
    PatDestructure,
}

impl Method {
    pub fn vec_from(impl_block: &ItemImpl) -> Result<Vec<Method>, MethodParseError> {
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
            .map(|fn_context| -> Result<Method, MethodParseError> {
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
                            Err(MethodParseError::PatDestructure)
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let return_type = match &fn_context.sig.output {
                    syn::ReturnType::Default => None,
                    syn::ReturnType::Type(_, t) => Some(t.into_token_stream().to_string()),
                };

                Ok(Method {
                    comments: None,
                    name,
                    args,
                    return_type,
                })
            })
            .collect()
    }
}
