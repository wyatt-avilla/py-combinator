use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::{Ident, Token};

use crate::{REGISTER_METHODS_ATTRIBUTE, SELF_GENERIC_ATTRIBUTE};

use itertools::{self, Itertools};
use syn::ItemImpl;

use crate::impl_block::{ImplBlock, ImplBlockParseError};

pub struct Arg {
    pub key: Ident,
    _eq_token: Token![=],
    pub value: Ident,
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

impl ImplBlock {
    pub fn parse_self_generic(impl_block: &ItemImpl) -> Result<String, ImplBlockParseError> {
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

        let arg: crate::self_generic::Arg = register_attr
            .parse_args()
            .map_err(|_| ImplBlockParseError::MalformedSelfFunctionMarker)?;

        if arg.key == SELF_GENERIC_ATTRIBUTE {
            Ok(arg.value.to_string())
        } else {
            Err(ImplBlockParseError::MissingSelfGeneric)
        }
    }
}
