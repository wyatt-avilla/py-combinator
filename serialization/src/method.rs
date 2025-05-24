use quote::ToTokens;
use serde::Serialize;
use syn::ItemImpl;

#[derive(Serialize)]
pub struct Method {
    comments: Option<String>,
    name: String,
    args: Vec<String>,
    return_type: Option<String>,
}

impl Method {
    pub fn vec_from(impl_block: &ItemImpl) -> Vec<Method> {
        impl_block
            .items
            .iter()
            .filter_map(|i| {
                if let syn::ImplItem::Fn(m) = i {
                    let name = m.sig.ident.to_string();
                    let args = m
                        .sig
                        .inputs
                        .iter()
                        .map(|arg| quote::quote! { #arg }.to_string())
                        .collect();
                    let return_type = match &m.sig.output {
                        syn::ReturnType::Default => None,
                        syn::ReturnType::Type(_, t) => Some(t.into_token_stream().to_string()),
                    };

                    Some(Method {
                        comments: None,
                        name,
                        args,
                        return_type,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
