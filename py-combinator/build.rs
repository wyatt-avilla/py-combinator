use itertools::{self, Itertools};
use serialization::{ImplBlock, REGISTER_METHODS_ATTRIBUTE, SERIALIZED_METHODS_PATH};
use std::fs;
use std::path::Path;
use syn::Item;
use walkdir::WalkDir;

macro_rules! log {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[2K\x1b[36;1m    build.rs\x1b[0m {}", format!($($tokens)*));
    }
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
                    let impl_block = ImplBlock::from(&impl_block).map_err(|e| e.to_string())?;
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

    fs::create_dir_all(Path::new(SERIALIZED_METHODS_PATH).parent().unwrap()).unwrap();
    fs::write(
        SERIALIZED_METHODS_PATH,
        serde_json::to_string_pretty(&impl_blocks).unwrap(),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=src/");
    Ok(())
}
