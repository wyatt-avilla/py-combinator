use crate::method::Method;

use itertools::{self, Itertools};
use serde::Serialize;
use syn::ItemImpl;
use thiserror::Error;

#[derive(Serialize)]
pub struct ImplBlock {
    name: Vec<String>,
    methods: Vec<Method>,
}

#[derive(Debug, Error)]
pub enum ImplBlockParseError {
    #[error("Couldn't destructure `ItemImpl` into `Type::Path`")]
    PathDestructure,

    #[error("Attempted to register methods without a fully qualified impl block path")]
    UnqualifiedPath,
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

            if name.iter().any(|n| n == "self") || !name.iter().any(|n| n == "crate") {
                return Err(ImplBlockParseError::UnqualifiedPath);
            }

            let methods = Method::vec_from(impl_block);

            Ok(ImplBlock { name, methods })
        } else {
            Err(ImplBlockParseError::PathDestructure)
        }
    }
}
