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

        impl #struct_name {
            pub const BASE_PATH: &'static str = #base_path_value;

            #constructor
        }
    };

    TokenStream::from(expanded)
}
