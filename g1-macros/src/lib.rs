//! G1 proc macros.

extern crate proc_macro;

use g1_common::proc_macro::query_proc_macro;
use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::quote;

#[proc_macro_hack]
pub fn query(input: TokenStream) -> TokenStream {
    let output = match query_proc_macro(input.into()) {
        Ok(toks) => toks,
        Err(err) => quote! { compile_error!(#err)},
    };
    output.into()
}
