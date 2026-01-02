use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, parse_macro_input};

pub fn validate_impl(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = quote! {
        {
            if !(#expr) {
                return Err(armature_core::Error::Validation("Validation failed".to_string()));
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn validate_required_impl(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = quote! {
        {
            if (#expr).is_none() {
                return Err(armature_core::Error::Validation("Required field is missing".to_string()));
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn validate_email_impl(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = quote! {
        {
            let email = #expr;
            let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
            if !email_regex.is_match(&email) {
                return Err(armature_core::Error::Validation("Invalid email format".to_string()));
            }
        }
    };

    TokenStream::from(expanded)
}
