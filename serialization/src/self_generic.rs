use crate::{
    SELF_GENERIC_ATTRIBUTE,
    attr_list::{AttributeArg, AttributeArgsList},
};

use itertools::{self, Itertools};
use syn::ItemImpl;

use crate::impl_block::{ImplBlock, ImplBlockParseError};

impl ImplBlock {
    pub fn parse_self_generic(impl_block: &ItemImpl) -> Result<String, ImplBlockParseError> {
        let register_attrs = impl_block
            .attrs
            .iter()
            .map(|attr| {
                attr.parse_args::<AttributeArgsList>()
                    .map_err(|e| ImplBlockParseError::AttributeParseError(e.to_string()))
            })
            .map_ok(|args| {
                args.0.into_iter().find_map(|a| {
                    if let AttributeArg::KeyValueArg(kv) = a {
                        if kv.key == SELF_GENERIC_ATTRIBUTE {
                            Some(kv)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .collect::<Result<Vec<_>, ImplBlockParseError>>()?
            .into_iter()
            .flatten()
            .collect_vec();

        if register_attrs.len() != 1 {
            return Err(ImplBlockParseError::NotExactlyOneSelfFunctionMarker);
        }

        let key = &register_attrs[0].key;
        let val = &register_attrs[0].value;

        if key == SELF_GENERIC_ATTRIBUTE {
            Ok(val.to_string())
        } else {
            Err(ImplBlockParseError::MissingSelfGeneric)
        }
    }
}
