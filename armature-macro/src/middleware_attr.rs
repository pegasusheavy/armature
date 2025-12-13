use proc_macro::TokenStream;
use quote::quote;
use syn::{
    FnArg, ItemFn, Type, parse::Parse, parse::ParseStream, parse_macro_input, punctuated::Punctuated,
    token::Comma,
};

/// Arguments for the use_middleware attribute
/// Parses: #[use_middleware(Middleware1, Middleware2, ...)]
struct UseMiddlewareArgs {
    middlewares: Punctuated<Type, Comma>,
}

impl Parse for UseMiddlewareArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            middlewares: Punctuated::parse_terminated(input)?,
        })
    }
}

/// Implementation of the `#[use_middleware(...)]` attribute macro.
///
/// This macro wraps a route handler function to apply middleware before the handler executes.
///
/// # Usage
///
/// ```ignore
/// use armature::{get, use_middleware};
/// use armature_core::{HttpRequest, HttpResponse, Error, LoggerMiddleware, CorsMiddleware};
///
/// #[use_middleware(LoggerMiddleware::new(), CorsMiddleware::new())]
/// #[get("/users")]
/// async fn get_users(req: HttpRequest) -> Result<HttpResponse, Error> {
///     Ok(HttpResponse::ok())
/// }
/// ```
///
/// # How It Works
///
/// The macro generates a wrapper function that:
/// 1. Creates a middleware chain with all specified middlewares
/// 2. Wraps the original handler function
/// 3. Applies the middleware chain to incoming requests
pub fn use_middleware_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as UseMiddlewareArgs);
    let input = parse_macro_input!(item as ItemFn);

    let func_name = &input.sig.ident;
    let func_vis = &input.vis;
    let func_attrs = &input.attrs;
    let func_output = &input.sig.output;
    let func_body = &input.block;
    let is_async = input.sig.asyncness.is_some();

    // Check if there's a self receiver
    let has_self_receiver = input.sig.inputs.first().map_or(false, |arg| {
        matches!(arg, FnArg::Receiver(_))
    });

    // Get the middlewares
    let middlewares: Vec<_> = args.middlewares.iter().collect();

    if middlewares.is_empty() {
        // No middlewares specified, return original function
        return TokenStream::from(quote! {
            #(#func_attrs)*
            #func_vis #input
        });
    }

    // Generate middleware chain setup
    let middleware_setup: Vec<_> = middlewares
        .iter()
        .map(|mw| {
            quote! {
                __middleware_chain.use_middleware(#mw);
            }
        })
        .collect();

    // Generate the appropriate function signature
    let async_marker = if is_async {
        quote! { async }
    } else {
        quote! {}
    };

    // Create the inner handler name
    let inner_fn_name = syn::Ident::new(
        &format!("__{}_inner", func_name),
        func_name.span(),
    );

    // Build the function inputs
    let func_inputs = &input.sig.inputs;

    // Check if the function takes HttpRequest
    let takes_request = func_inputs.iter().any(|arg| {
        if let FnArg::Typed(pat_type) = arg {
            if let Type::Path(type_path) = pat_type.ty.as_ref() {
                if let Some(segment) = type_path.path.segments.last() {
                    return segment.ident == "HttpRequest";
                }
            }
        }
        false
    });

    let expanded = if has_self_receiver {
        // For methods with self receiver
        // This is more complex as we need to handle the self parameter
        quote! {
            #(#func_attrs)*
            #func_vis #async_marker fn #func_name(&self, __request: armature_core::HttpRequest) #func_output {
                use armature_core::middleware::{MiddlewareChain, Middleware};
                use std::sync::Arc;

                // Create middleware chain
                let mut __middleware_chain = MiddlewareChain::new();
                #(#middleware_setup)*

                // Create the inner handler
                let __inner_handler: armature_core::middleware::HandlerFn = Arc::new(|__req: armature_core::HttpRequest| {
                    Box::pin(async move {
                        // Execute the original function body
                        #func_body
                    }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<armature_core::HttpResponse, armature_core::Error>> + Send>>
                });

                // Apply middleware chain
                __middleware_chain.apply(__request, __inner_handler).await
            }
        }
    } else if takes_request {
        // For standalone functions that take HttpRequest
        quote! {
            #(#func_attrs)*
            #func_vis #async_marker fn #func_name(__request: armature_core::HttpRequest) #func_output {
                use armature_core::middleware::{MiddlewareChain, Middleware};
                use std::sync::Arc;

                // Create middleware chain
                let mut __middleware_chain = MiddlewareChain::new();
                #(#middleware_setup)*

                // Create the inner handler
                let __inner_handler: armature_core::middleware::HandlerFn = Arc::new(|__req: armature_core::HttpRequest| {
                    Box::pin(async move {
                        // Define the original function body as inner
                        #async_marker fn #inner_fn_name(__request: armature_core::HttpRequest) #func_output
                            #func_body

                        #inner_fn_name(__req).await
                    }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<armature_core::HttpResponse, armature_core::Error>> + Send>>
                });

                // Apply middleware chain
                __middleware_chain.apply(__request, __inner_handler).await
            }
        }
    } else {
        // For functions that don't take HttpRequest (using extractors)
        quote! {
            #(#func_attrs)*
            #func_vis #async_marker fn #func_name(__request: armature_core::HttpRequest) #func_output {
                use armature_core::middleware::{MiddlewareChain, Middleware};
                use std::sync::Arc;

                // Create middleware chain
                let mut __middleware_chain = MiddlewareChain::new();
                #(#middleware_setup)*

                // Create the inner handler
                let __inner_handler: armature_core::middleware::HandlerFn = Arc::new(|__req: armature_core::HttpRequest| {
                    Box::pin(async move {
                        // Execute the original function body
                        #func_body
                    }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<armature_core::HttpResponse, armature_core::Error>> + Send>>
                });

                // Apply middleware chain
                __middleware_chain.apply(__request, __inner_handler).await
            }
        }
    };

    TokenStream::from(expanded)
}

/// Implementation for applying middleware to an entire controller/module
pub fn middleware_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as UseMiddlewareArgs);
    let input: syn::Item = parse_macro_input!(item);

    match input {
        syn::Item::Struct(item_struct) => {
            // For structs (controllers), we store middleware metadata
            let struct_name = &item_struct.ident;
            let vis = &item_struct.vis;
            let attrs = &item_struct.attrs;
            let fields = &item_struct.fields;
            let generics = &item_struct.generics;

            let middlewares: Vec<_> = args.middlewares.iter().collect();

            let middleware_const_name = syn::Ident::new(
                &format!("__MIDDLEWARES_{}", struct_name.to_string().to_uppercase()),
                struct_name.span(),
            );

            // Generate middleware factory function
            let middleware_factories: Vec<_> = middlewares
                .iter()
                .enumerate()
                .map(|(i, mw)| {
                    let factory_name = syn::Ident::new(
                        &format!("__middleware_factory_{}", i),
                        proc_macro2::Span::call_site(),
                    );
                    quote! {
                        fn #factory_name() -> Box<dyn armature_core::middleware::Middleware> {
                            Box::new(#mw)
                        }
                    }
                })
                .collect();

            let middleware_count = middlewares.len();

            quote! {
                #(#attrs)*
                #vis struct #struct_name #generics #fields

                impl #struct_name {
                    /// Get the middleware factories for this controller
                    pub fn __get_middleware_factories() -> Vec<fn() -> Box<dyn armature_core::middleware::Middleware>> {
                        vec![#(Self::#middleware_factories),*]
                    }

                    #(#middleware_factories)*
                }

                /// Number of middlewares for this controller
                pub const #middleware_const_name: usize = #middleware_count;
            }
            .into()
        }
        syn::Item::Fn(item_fn) => {
            // For functions, use the function implementation
            let middlewares: Vec<_> = args.middlewares.iter().collect();
            let tokens: proc_macro2::TokenStream = quote! { #(#middlewares),* };
            use_middleware_impl(tokens.into(), quote! { #item_fn }.into())
        }
        _ => {
            syn::Error::new_spanned(
                quote! {},
                "use_middleware can only be applied to functions or structs",
            )
            .to_compile_error()
            .into()
        }
    }
}

