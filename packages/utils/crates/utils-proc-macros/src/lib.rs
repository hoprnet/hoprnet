#![doc(html_root_url = "https://docs.rs/wasm-bindgen-macro/0.2")]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

/// Small wrapper around `wasm-bindgen` proc_macro to build wasm
/// binding code only if `feature = "wasm"` is activated
#[proc_macro_attribute]
pub fn wasm_bindgen_if(attr: TokenStream, input: TokenStream) -> TokenStream {
    if cfg!(feature = "wasm") {
        match wasm_bindgen_macro_support::expand(attr.into(), input.into()) {
            Ok(tokens) => {
                if cfg!(feature = "xxx_debug_only_print_generated_code") {
                    println!("{}", tokens);
                }
                tokens.into()
            }
            Err(diagnostic) => (quote! { #diagnostic }).into(),
        }
    } else {
        input
    }
}
