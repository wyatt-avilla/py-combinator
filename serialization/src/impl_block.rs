use crate::method::{self, Method};

use itertools::{self, Itertools};
use serde::Serialize;
use syn::ItemImpl;

use thiserror::Error;

#[derive(Serialize, Debug)]
pub struct ImplBlock {
    name: Vec<String>,
    self_function: String,
    self_generic: String,
    methods: Vec<Method>,
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

            let methods =
                Method::vec_from(impl_block).map_err(ImplBlockParseError::MethodParseError)?;

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
}
