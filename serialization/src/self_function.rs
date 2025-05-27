use crate::impl_block::{ImplBlock, ImplBlockParseError};

use syn::{ImplItem, ItemImpl, Meta};

impl ImplBlock {
    pub fn parse_self_function(impl_block: &ItemImpl) -> Result<String, ImplBlockParseError> {
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
            .is_some_and(|a| matches!(a, syn::FnArg::Receiver(_)))
        {
            Ok(self_function_vec[0].ident.to_string())
        } else {
            Err(ImplBlockParseError::MalformedSelfFunctionMarker)
        }
    }
}
