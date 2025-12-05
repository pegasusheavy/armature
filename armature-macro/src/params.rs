use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn body_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn from_request(request: &armature_core::HttpRequest) -> Result<Self, armature_core::Error> {
                request.json::<Self>()
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn param_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn from_param(request: &armature_core::HttpRequest, param_name: &str) -> Result<Self, armature_core::Error> {
                let value = request.param(param_name)
                    .ok_or_else(|| armature_core::Error::Validation(format!("Missing parameter: {}", param_name)))?;

                value.parse::<Self>()
                    .map_err(|_| armature_core::Error::Validation(format!("Invalid parameter value: {}", value)))
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn query_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn from_query(request: &armature_core::HttpRequest) -> Result<Self, armature_core::Error> {
                // For now, we'll implement a simple version
                // In a real implementation, this would use serde to deserialize from query params
                Err(armature_core::Error::Internal("Query parameter extraction not yet fully implemented".to_string()))
            }
        }
    };

    TokenStream::from(expanded)
}
