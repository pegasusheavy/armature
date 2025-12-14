use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn derive_model_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            /// Create a new instance
            pub fn new() -> Self {
                Self::default()
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn derive_api_model_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            /// Convert to JSON value
            pub fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
                serde_json::to_value(self)
            }

            /// Convert from JSON value
            pub fn from_json(value: &serde_json::Value) -> Result<Self, serde_json::Error> {
                serde_json::from_value(value.clone())
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn derive_resource_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            /// Get the table name for this resource
            pub fn table_name() -> &'static str {
                // Default table name based on struct name
                stringify!(#name)
            }
        }
    };

    TokenStream::from(expanded)
}

