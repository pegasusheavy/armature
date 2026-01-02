use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for extracting request body
///
/// Implements `FromRequest` for the type, allowing it to be extracted from
/// the request body as JSON.
///
/// # Example
///
/// ```rust,ignore
/// use armature::prelude::*;
///
/// #[derive(Body, Deserialize)]
/// struct CreateUser {
///     name: String,
///     email: String,
/// }
///
/// // In a handler
/// let user: CreateUser = body!(request)?;
/// // Or using the extractor
/// let body: Body<CreateUser> = Body::from_request(&request)?;
/// ```
pub fn body_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Extract this type from the request body as JSON
            pub fn from_request(request: &armature_core::HttpRequest) -> Result<Self, armature_core::Error> {
                request.json::<Self>()
            }

            /// Extract this type from the request body as JSON, returning Option
            pub fn from_request_opt(request: &armature_core::HttpRequest) -> Option<Self> {
                request.json::<Self>().ok()
            }
        }

        impl #impl_generics armature_core::extractors::FromRequest for #name #ty_generics #where_clause {
            fn from_request(request: &armature_core::HttpRequest) -> Result<Self, armature_core::Error> {
                request.json::<Self>()
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for extracting path parameters
///
/// Implements parsing from a string for single-value types, or from multiple
/// path parameters for structs.
///
/// # Example
///
/// ```rust,ignore
/// // For single values (implementing FromStr)
/// #[derive(Param)]
/// struct UserId(u32);
///
/// // Usage: let id: UserId = path!(request, "id")?;
///
/// // For structs with multiple path params
/// #[derive(Param, Deserialize)]
/// struct UserPostParams {
///     user_id: u32,
///     post_id: u32,
/// }
///
/// // Usage for /users/:user_id/posts/:post_id
/// let params: UserPostParams = PathParams::from_request(&request)?;
/// ```
pub fn param_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Extract a path parameter by name
            pub fn from_param(request: &armature_core::HttpRequest, param_name: &str) -> Result<Self, armature_core::Error>
            where
                Self: std::str::FromStr,
                <Self as std::str::FromStr>::Err: std::fmt::Display,
            {
                let value = request.param(param_name)
                    .ok_or_else(|| armature_core::Error::Validation(format!("Missing parameter: {}", param_name)))?;

                value.parse::<Self>()
                    .map_err(|e| armature_core::Error::Validation(format!("Invalid parameter '{}': {}", param_name, e)))
            }

            /// Extract a path parameter by name, returning Option
            pub fn from_param_opt(request: &armature_core::HttpRequest, param_name: &str) -> Option<Self>
            where
                Self: std::str::FromStr,
            {
                request.param(param_name)
                    .and_then(|v| v.parse::<Self>().ok())
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for extracting query parameters
///
/// Implements `FromRequest` for the type, allowing it to be extracted from
/// URL query parameters.
///
/// # Example
///
/// ```rust,ignore
/// use armature::prelude::*;
///
/// #[derive(Query, Deserialize)]
/// struct UserFilters {
///     page: Option<u32>,
///     limit: Option<u32>,
///     sort: Option<String>,
///     order: Option<String>,
/// }
///
/// // In a handler
/// let filters: UserFilters = query!(request)?;
/// // Or using the extractor
/// let query: Query<UserFilters> = Query::from_request(&request)?;
/// ```
pub fn query_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Extract this type from query parameters
            pub fn from_query(request: &armature_core::HttpRequest) -> Result<Self, armature_core::Error>
            where
                Self: serde::de::DeserializeOwned,
            {
                let query_string: String = request.query_params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&");

                serde_urlencoded::from_str(&query_string)
                    .map_err(|e| armature_core::Error::Validation(format!("Invalid query parameters: {}", e)))
            }

            /// Extract this type from query parameters, returning Option
            pub fn from_query_opt(request: &armature_core::HttpRequest) -> Option<Self>
            where
                Self: serde::de::DeserializeOwned,
            {
                Self::from_query(request).ok()
            }
        }

        impl #impl_generics armature_core::extractors::FromRequest for #name #ty_generics #where_clause
        where
            Self: serde::de::DeserializeOwned,
        {
            fn from_request(request: &armature_core::HttpRequest) -> Result<Self, armature_core::Error> {
                Self::from_query(request)
            }
        }
    };

    TokenStream::from(expanded)
}
