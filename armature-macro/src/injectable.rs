use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, parse_macro_input};

pub fn injectable_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    // The #[injectable] attribute is now a marker that the type can be used with DI.
    // Since `Provider` is a blanket impl for `Send + Sync + 'static` types,
    // we no longer need to generate an explicit impl.
    // The macro now just passes through the struct definition.
    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}
