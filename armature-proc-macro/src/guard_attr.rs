use proc_macro::TokenStream;
use quote::quote;
use syn::{
    FnArg, ItemFn, Type, parse::Parse, parse::ParseStream, parse_macro_input, punctuated::Punctuated,
    token::Comma,
};

/// Arguments for the use_guard attribute
/// Parses: #[use_guard(Guard1, Guard2, ...)]
struct UseGuardArgs {
    guards: Punctuated<Type, Comma>,
}

impl Parse for UseGuardArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            guards: Punctuated::parse_terminated(input)?,
        })
    }
}

/// Implementation of the `#[use_guard(...)]` attribute macro.
///
/// This macro wraps a route handler function to check guards before execution.
///
/// # Usage
///
/// ```ignore
/// use armature::{get, use_guard};
/// use armature_core::{HttpRequest, HttpResponse, Error, AuthenticationGuard};
///
/// #[use_guard(AuthenticationGuard)]
/// #[get("/protected")]
/// async fn protected_endpoint(req: HttpRequest) -> Result<HttpResponse, Error> {
///     Ok(HttpResponse::ok())
/// }
/// ```
///
/// # How It Works
///
/// The macro generates a wrapper function that:
/// 1. Creates instances of all specified guards
/// 2. Runs each guard's `can_activate` method
/// 3. If all guards pass, executes the original handler
/// 4. If any guard fails, returns the error response
pub fn use_guard_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as UseGuardArgs);
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

    // Get the guards
    let guards: Vec<_> = args.guards.iter().collect();

    if guards.is_empty() {
        // No guards specified, return original function
        return TokenStream::from(quote! {
            #(#func_attrs)*
            #func_vis #input
        });
    }

    // Generate guard checks
    let guard_checks: Vec<_> = guards
        .iter()
        .map(|guard| {
            quote! {
                {
                    let __guard: #guard = Default::default();
                    let __context = armature_core::guard::GuardContext::new(__request.clone());
                    match __guard.can_activate(&__context).await {
                        Ok(true) => {},
                        Ok(false) => {
                            return Err(armature_core::Error::Forbidden(
                                "Access denied by guard".to_string()
                            ));
                        },
                        Err(e) => return Err(e),
                    }
                }
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
        quote! {
            #(#func_attrs)*
            #func_vis #async_marker fn #func_name(&self, __request: armature_core::HttpRequest) #func_output {
                use armature_core::guard::Guard;

                // Run all guards
                #(#guard_checks)*

                // All guards passed, execute handler
                #func_body
            }
        }
    } else if takes_request {
        // For standalone functions that take HttpRequest
        quote! {
            #(#func_attrs)*
            #func_vis #async_marker fn #func_name(__request: armature_core::HttpRequest) #func_output {
                use armature_core::guard::Guard;

                // Run all guards
                #(#guard_checks)*

                // All guards passed, define and call inner function
                #async_marker fn #inner_fn_name(__request: armature_core::HttpRequest) #func_output
                    #func_body

                #inner_fn_name(__request).await
            }
        }
    } else {
        // For functions that don't take HttpRequest (using extractors)
        quote! {
            #(#func_attrs)*
            #func_vis #async_marker fn #func_name(__request: armature_core::HttpRequest) #func_output {
                use armature_core::guard::Guard;

                // Run all guards
                #(#guard_checks)*

                // All guards passed, execute handler
                #func_body
            }
        }
    };

    TokenStream::from(expanded)
}

/// Implementation for use_guard with guard instances (not just types)
/// Parses: #[guard(AuthGuard::new(), RolesGuard::new(vec!["admin"]))]
pub fn guard_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as UseGuardArgs);
    let input: syn::Item = parse_macro_input!(item);

    match input {
        syn::Item::Struct(item_struct) => {
            // For structs (controllers), store guard metadata
            let struct_name = &item_struct.ident;
            let vis = &item_struct.vis;
            let attrs = &item_struct.attrs;
            let fields = &item_struct.fields;
            let generics = &item_struct.generics;

            let guards: Vec<_> = args.guards.iter().collect();

            let guard_const_name = syn::Ident::new(
                &format!("__GUARDS_{}", struct_name.to_string().to_uppercase()),
                struct_name.span(),
            );

            // Generate guard factory functions
            let guard_factories: Vec<_> = guards
                .iter()
                .enumerate()
                .map(|(i, guard)| {
                    let factory_name = syn::Ident::new(
                        &format!("__guard_factory_{}", i),
                        proc_macro2::Span::call_site(),
                    );
                    quote! {
                        fn #factory_name() -> Box<dyn armature_core::guard::Guard> {
                            Box::new(#guard)
                        }
                    }
                })
                .collect();

            let guard_count = guards.len();

            quote! {
                #(#attrs)*
                #vis struct #struct_name #generics #fields

                impl #struct_name {
                    /// Get the guard factories for this controller
                    pub fn __get_guard_factories() -> Vec<fn() -> Box<dyn armature_core::guard::Guard>> {
                        vec![#(Self::#guard_factories),*]
                    }

                    #(#guard_factories)*
                }

                /// Number of guards for this controller
                pub const #guard_const_name: usize = #guard_count;
            }
            .into()
        }
        syn::Item::Fn(item_fn) => {
            // For functions, use the function implementation with guard instances
            let guards: Vec<_> = args.guards.iter().collect();
            let tokens: proc_macro2::TokenStream = quote! { #(#guards),* };
            use_guard_instance_impl(tokens.into(), quote! { #item_fn }.into())
        }
        _ => {
            syn::Error::new_spanned(
                quote! {},
                "guard can only be applied to functions or structs",
            )
            .to_compile_error()
            .into()
        }
    }
}

/// Implementation for guards with instances (constructed guards)
/// This allows: #[guard(AuthGuard::new(), RolesGuard::new(vec!["admin"]))]
fn use_guard_instance_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as UseGuardArgs);
    let input = parse_macro_input!(item as ItemFn);

    let func_name = &input.sig.ident;
    let func_vis = &input.vis;
    let func_attrs = &input.attrs;
    let func_output = &input.sig.output;
    let func_body = &input.block;
    let is_async = input.sig.asyncness.is_some();

    let has_self_receiver = input.sig.inputs.first().map_or(false, |arg| {
        matches!(arg, FnArg::Receiver(_))
    });

    let guards: Vec<_> = args.guards.iter().collect();

    if guards.is_empty() {
        return TokenStream::from(quote! {
            #(#func_attrs)*
            #func_vis #input
        });
    }

    // Generate guard checks with instances
    let guard_checks: Vec<_> = guards
        .iter()
        .map(|guard| {
            quote! {
                {
                    let __guard = #guard;
                    let __context = armature_core::guard::GuardContext::new(__request.clone());
                    match __guard.can_activate(&__context).await {
                        Ok(true) => {},
                        Ok(false) => {
                            return Err(armature_core::Error::Forbidden(
                                "Access denied by guard".to_string()
                            ));
                        },
                        Err(e) => return Err(e),
                    }
                }
            }
        })
        .collect();

    let async_marker = if is_async {
        quote! { async }
    } else {
        quote! {}
    };

    let inner_fn_name = syn::Ident::new(
        &format!("__{}_inner", func_name),
        func_name.span(),
    );

    let func_inputs = &input.sig.inputs;

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
        quote! {
            #(#func_attrs)*
            #func_vis #async_marker fn #func_name(&self, __request: armature_core::HttpRequest) #func_output {
                use armature_core::guard::Guard;

                #(#guard_checks)*

                #func_body
            }
        }
    } else if takes_request {
        quote! {
            #(#func_attrs)*
            #func_vis #async_marker fn #func_name(__request: armature_core::HttpRequest) #func_output {
                use armature_core::guard::Guard;

                #(#guard_checks)*

                #async_marker fn #inner_fn_name(__request: armature_core::HttpRequest) #func_output
                    #func_body

                #inner_fn_name(__request).await
            }
        }
    } else {
        quote! {
            #(#func_attrs)*
            #func_vis #async_marker fn #func_name(__request: armature_core::HttpRequest) #func_output {
                use armature_core::guard::Guard;

                #(#guard_checks)*

                #func_body
            }
        }
    };

    TokenStream::from(expanded)
}


