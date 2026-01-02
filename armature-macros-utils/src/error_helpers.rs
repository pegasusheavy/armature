use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, parse_macro_input};

pub fn bail_impl(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = quote! {
        return Err(armature_core::Error::BadRequest(#expr.to_string()))
    };

    TokenStream::from(expanded)
}

pub fn ensure_impl(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = quote! {
        if !(#expr) {
            return Err(armature_core::Error::BadRequest("Condition failed".to_string()));
        }
    };

    TokenStream::from(expanded)
}
