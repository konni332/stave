use darling::FromDeriveInput;
use proc_macro::TokenStream;
use syn::{DeriveInput, ItemImpl, parse_macro_input};

use crate::builder::{StaveBuilderInput, expand_builder};

mod builder;
mod common;
mod methods;

#[proc_macro_attribute]
pub fn builder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let parsed = match StaveBuilderInput::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    match expand_builder(parsed) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn methods(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    match methods::parse_and_expand(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
