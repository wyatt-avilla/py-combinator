use crate::method::{self, Method};
use crate::{REGISTER_METHODS_ATTRIBUTE, SELF_GENERIC_ATTRIBUTE};

use itertools::{self, Itertools};
use serde::Serialize;
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::{Ident, Token};
use syn::{ImplItem, ItemImpl, Meta};

use thiserror::Error;

#[derive(Serialize, Debug)]
pub struct ImplBlock {
    name: Vec<String>,
    self_function: String,
    self_generic: String,
    methods: Vec<Method>,
}

struct Arg {
    key: Ident,
    _eq_token: Token![=],
    value: Ident,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        Ok(Self {
            key: input.parse()?,
            _eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

#[derive(Debug, Error)]
pub enum ImplBlockParseError {
    #[error("Couldn't destructure `ItemImpl` into `Type::Path`")]
    PathDestructure,

    #[error("Didn't find exactly one `method_self_arg` attribute")]
    MultipleSelfFunctionMarkers,

    #[error("`method_self_arg` attribute is malformed")]
    MalformedSelfFunctionMarker,

    #[error("Couldn't find Self generic parameter")]
    MissingSelfGeneric,

    #[error("Couldn't parse one of the methods")]
    MethodParseError(method::MethodParseError),
}

// lol
impl ImplBlock {
    pub fn nice_name(&self) -> String {
        self.name.iter().join("::")
    }

    fn parse_self_generic(impl_block: &ItemImpl) -> Result<String, ImplBlockParseError> {
        let register_attr = impl_block
            .attrs
            .iter()
            .find(|attr| {
                attr.path().is_ident(REGISTER_METHODS_ATTRIBUTE)
                    || (attr
                        .path()
                        .segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .contains(REGISTER_METHODS_ATTRIBUTE))
            })
            .ok_or(ImplBlockParseError::MissingSelfGeneric)?;

        let arg: Arg = register_attr
            .parse_args()
            .map_err(|_| ImplBlockParseError::MalformedSelfFunctionMarker)?;

        if arg.key == SELF_GENERIC_ATTRIBUTE {
            Ok(arg.value.to_string())
        } else {
            Err(ImplBlockParseError::MissingSelfGeneric)
        }
    }

    pub fn from(impl_block: &ItemImpl) -> Result<ImplBlock, ImplBlockParseError> {
        if let syn::Type::Path(p) = *impl_block.clone().self_ty {
            let name: Vec<_> = p
                .path
                .segments
                .iter()
                .map(|x| x.clone().ident.to_string())
                .collect();

            let self_function_vec = impl_block
                .items
                .iter()
                .filter_map(|x| match x {
                    ImplItem::Fn(f) => Some(f.clone()),
                    _ => None,
                })
                .filter_map(|i| {
                    let path = i.attrs.clone().first().cloned().and_then(|a| match a.meta {
                        Meta::Path(p) => Some(p.segments),
                        _ => None,
                    });

                    if path.is_some_and(|p| {
                        p.into_iter()
                            .map(|p| p.ident)
                            .any(|i| i == crate::SELF_FUNC_ATTRIBUTE)
                    }) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .map(|i| i.sig)
                .collect::<Vec<_>>();

            if self_function_vec.len() != 1 {
                return Err(ImplBlockParseError::MultipleSelfFunctionMarkers);
            }

            if self_function_vec[0]
                .inputs
                .first()
                .is_none_or(|a| !matches!(a, syn::FnArg::Receiver(_)))
            {
                return Err(ImplBlockParseError::MalformedSelfFunctionMarker);
            }

            let self_generic = ImplBlock::parse_self_generic(impl_block)?;

            let self_function = self_function_vec[0].ident.to_string();

            let methods = Method::vec_from(impl_block);

            Ok(ImplBlock {
                name,
                self_function,
                self_generic,
                methods: methods.map_err(ImplBlockParseError::MethodParseError)?,
            })
        } else {
            Err(ImplBlockParseError::PathDestructure)
        }
    }
}
