use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro]
pub fn my_macro(_input: TokenStream) -> TokenStream {
    let output = quote! {
        fn generated_function() {
            println!("Hello from a proc macro!");
        }
    };
    output.into()
}
