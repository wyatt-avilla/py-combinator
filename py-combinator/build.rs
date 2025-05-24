use itertools::{self, Itertools};
use quote::ToTokens;
use serde::Serialize;
use syn::{Item, ItemImpl};
use thiserror::Error;
use walkdir::WalkDir;

macro_rules! log {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[2K\x1b[36;1m    build.rs\x1b[0m {}", format!($($tokens)*));
    }
}

const REGISTER_METHODS_ATTRIBUTE: &str = "register_methods";

#[derive(Serialize)]
struct ImplBlock {
    name: Vec<String>,
    methods: Vec<Method>,
}

impl ImplBlock {
    fn nice_name(&self) -> String {
        self.name.iter().join("::")
    }
}

#[derive(Serialize)]
struct Method {
    comments: Option<String>,
    name: String,
    args: Vec<String>,
    return_type: Option<String>,
}

#[derive(Debug, Error)]
enum ImplBlockParseError {
    #[error("Couldn't destructure `ItemImpl` into `Type::Path`")]
    PathDestructure,

    #[error("Attempted to register methods without a fully qualified impl block path")]
    UnqualifiedPath,
}

fn parse_impl_block(impl_block: &ItemImpl) -> Result<ImplBlock, ImplBlockParseError> {
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

        let methods = parse_methods(impl_block);

        Ok(ImplBlock { name, methods })
    } else {
        Err(ImplBlockParseError::PathDestructure)
    }
}

fn parse_methods(impl_block: &ItemImpl) -> Vec<Method> {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut impl_blocks: Vec<ImplBlock> = Vec::new();

    for entry in WalkDir::new("src") {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }

        let src = std::fs::read_to_string(path)?;
        let file = syn::parse_file(&src)?;

        for item in file.items {
            if let Item::Impl(impl_block) = item {
                let has_marker = impl_block.attrs.iter().any(|a| {
                    a.path()
                        .segments
                        .iter()
                        .any(|s| s.ident == REGISTER_METHODS_ATTRIBUTE)
                });
                if has_marker {
                    let impl_block = parse_impl_block(&impl_block).map_err(|e| e.to_string())?;
                    impl_blocks.push(impl_block);
                }
            }
        }
    }

    assert!(
        !impl_blocks.is_empty(),
        "No #[{REGISTER_METHODS_ATTRIBUTE}] impl blocks found",
    );

    log!(
        "parsed {} impl block{} [{}]",
        impl_blocks.len(),
        if impl_blocks.len() == 1 { "" } else { "s" },
        impl_blocks.iter().map(ImplBlock::nice_name).join(", ")
    );

    std::fs::write(
        "target/iterator_methods.json",
        serde_json::to_string_pretty(&impl_blocks).unwrap(),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=src/");
    Ok(())
}
