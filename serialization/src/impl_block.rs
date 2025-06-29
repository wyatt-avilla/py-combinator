use crate::method::Method;

use itertools::{self, Itertools};
use serde::{Deserialize, Serialize};
use syn::{ImplItem, ItemImpl, Meta};

use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct ImplBlock {
    pub name: Vec<String>,
    pub self_function: String,
    pub self_generic: String,
    pub methods: Vec<Method>,
}

#[derive(Debug, Error)]
pub enum ImplBlockParseError {
    #[error("Couldn't destructure `ItemImpl` into `Type::Path`")]
    PathDestructure,

    #[error("Didn't find exactly one `method_self_arg` attribute")]
    NotExactlyOneSelfFunctionMarker,

    #[error("`method_self_arg` attribute is malformed")]
    MalformedSelfFunctionMarker,

    #[error("Couldn't find Self generic parameter")]
    MissingSelfGeneric,

    #[error("Couldn't parse one of attribute blocks")]
    AttributeParseError(String),

    #[error("Couldn't destructure `PatType` into `Pat::Ident`")]
    PatDestructure,
}

// lol
impl ImplBlock {
    pub fn nice_name(&self) -> String {
        self.name.iter().join("::")
    }

    pub fn from(impl_block: &ItemImpl) -> Result<ImplBlock, ImplBlockParseError> {
        if let syn::Type::Path(p) = *impl_block.clone().self_ty {
            let name: Vec<_> = p
                .path
                .segments
                .iter()
                .map(|x| x.clone().ident.to_string())
                .collect();

            let self_generic = ImplBlock::parse_self_generic(impl_block)?;

            let self_function = ImplBlock::parse_self_function(impl_block)?;

            let methods = Method::vec_from(impl_block)?;

            Ok(ImplBlock {
                name,
                self_function,
                self_generic,
                methods,
            })
        } else {
            Err(ImplBlockParseError::PathDestructure)
        }
    }

    pub fn find_method_with_attribute_containing(
        impl_block: &ItemImpl,
        attr_query: &str,
    ) -> Vec<(syn::ImplItemFn, syn::Attribute)> {
        impl_block
            .items
            .iter()
            .filter_map(|x| match x {
                ImplItem::Fn(f) => Some(f.clone()),
                _ => None,
            })
            .flat_map(|item_fn| {
                item_fn
                    .attrs
                    .clone()
                    .into_iter()
                    .filter_map(|a| match a.meta.clone() {
                        Meta::Path(p) => Some((p.segments, a)),
                        Meta::List(l) => Some((l.path.segments, a)),
                        Meta::NameValue(_) => None,
                    })
                    .filter_map(move |(p, attr)| {
                        if p.into_iter().map(|p| p.ident).any(|i| i == attr_query) {
                            Some((item_fn.clone(), attr))
                        } else {
                            None
                        }
                    })
            })
            .collect()
    }
}
