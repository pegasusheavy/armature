use proc_macro::TokenStream;
use quote::quote;
use syn::{Fields, ItemStruct, LitStr, parse_macro_input};

pub fn controller_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    let base_path = parse_macro_input!(attr as LitStr);
    let base_path_value = base_path.value();

    // Extract field types for dependency injection
    let field_types: Vec<_> = match &input.fields {
        Fields::Named(fields) => fields.named.iter().map(|f| &f.ty).collect(),
        _ => vec![],
    };

    let field_names: Vec<_> = match &input.fields {
        Fields::Named(fields) => fields.named.iter().map(|f| &f.ident).collect(),
        _ => vec![],
    };

    // Generate constructor that resolves dependencies
    let constructor = if !field_types.is_empty() {
        quote! {
            pub fn new_with_di(container: &armature_core::Container) -> Result<Self, armature_core::Error> {
                Ok(Self {
                    #(#field_names: (*container.resolve::<#field_types>()?).clone()),*
                })
            }
        }
    } else {
        quote! {
            pub fn new_with_di(_container: &armature_core::Container) -> Result<Self, armature_core::Error> {
                Ok(Self::default())
            }
        }
    };

    let expanded = quote! {
        #input

        impl armature_core::Provider for #struct_name {}

        #[async_trait::async_trait]
        impl armature_core::Controller for #struct_name {
            fn base_path(&self) -> &'static str {
                #base_path_value
            }

            fn routes(&self) -> Vec<armature_core::RouteDefinition> {
                Self::__collect_routes()
            }
        }

        impl #struct_name {
            pub const BASE_PATH: &'static str = #base_path_value;

            #constructor

            /// Collect all routes defined on this controller.
            /// This method is called by the Controller trait implementation.
            /// Routes are added via the route registration macros.
            pub fn __collect_routes() -> Vec<armature_core::RouteDefinition> {
                // This will be populated by route macros via inventory or manual registration
                // For now, return empty - routes are registered via module's route_registrar
                vec![]
            }
        }
    };

    TokenStream::from(expanded)
}
