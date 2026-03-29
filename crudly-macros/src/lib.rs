extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Crudly)]
pub fn derive_crudly(input: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(input as DeriveInput);

    // match 

    "fn answer() -> u32 { 42 }".parse().unwrap()
}