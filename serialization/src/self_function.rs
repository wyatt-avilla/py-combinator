use crate::{
    SELF_FUNC_ATTRIBUTE,
    impl_block::{ImplBlock, ImplBlockParseError},
};

use syn::ItemImpl;

impl ImplBlock {
    pub fn parse_self_function(impl_block: &ItemImpl) -> Result<String, ImplBlockParseError> {
        let self_function_vec =
            ImplBlock::find_method_with_attribute_containing(impl_block, SELF_FUNC_ATTRIBUTE);

        if self_function_vec.len() != 1 {
            return Err(ImplBlockParseError::NotExactlyOneSelfFunctionMarker);
        }

        if self_function_vec[0]
            .sig
            .inputs
            .first()
            .is_some_and(|a| matches!(a, syn::FnArg::Receiver(_)))
        {
            Ok(self_function_vec[0].sig.ident.to_string())
        } else {
            Err(ImplBlockParseError::MalformedSelfFunctionMarker)
        }
    }
}
