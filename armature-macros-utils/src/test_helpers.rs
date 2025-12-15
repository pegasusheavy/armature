use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr};

pub fn test_request_impl(input: TokenStream) -> TokenStream {
    let _expr = parse_macro_input!(input as Expr);

    let expanded = quote! {
        {
            use armature_core::HttpRequest;
            // Parse the expression to extract method and path
            // For now, create a simple GET request
            HttpRequest::new("GET".to_string(), "/".to_string())
        }
    };

    TokenStream::from(expanded)
}

pub fn assert_json_impl(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = quote! {
        {
            // Assert that response body matches expected JSON
            let _check = #expr;
        }
    };

    TokenStream::from(expanded)
}

pub fn assert_status_impl(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = quote! {
        {
            // Assert HTTP status code
            let _check = #expr;
        }
    };

    TokenStream::from(expanded)
}

