use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr};

pub fn json_impl(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    // Simple implementation: wrap in HttpResponse::ok().with_json()
    let expanded = quote! {
        {
            use armature_core::{HttpResponse, Error};
            HttpResponse::ok().with_json(&#expr)
        }
    };

    TokenStream::from(expanded)
}

pub fn html_impl(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = quote! {
        {
            use armature_core::HttpResponse;
            let mut response = HttpResponse::ok();
            response.body = (#expr).to_string().into_bytes();
            response.headers.insert(
                "Content-Type".to_string(),
                "text/html; charset=utf-8".to_string()
            );
            Ok(response)
        }
    };

    TokenStream::from(expanded)
}

pub fn text_impl(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = quote! {
        {
            use armature_core::HttpResponse;
            let mut response = HttpResponse::ok();
            response.body = (#expr).to_string().into_bytes();
            response.headers.insert(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string()
            );
            Ok(response)
        }
    };

    TokenStream::from(expanded)
}

pub fn redirect_impl(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = quote! {
        {
            use armature_core::HttpResponse;
            let mut response = HttpResponse::new(302);
            response.headers.insert(
                "Location".to_string(),
                (#expr).to_string()
            );
            Ok(response)
        }
    };

    TokenStream::from(expanded)
}

