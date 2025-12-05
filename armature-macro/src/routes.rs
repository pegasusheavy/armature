use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, LitStr, parse_macro_input};

pub fn route_impl(attr: TokenStream, item: TokenStream, method: &str) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;
    let func_inputs = &input.sig.inputs;
    let func_output = &input.sig.output;
    let func_body = &input.block;
    let func_vis = &input.vis;
    let func_attrs = &input.attrs;

    let path = if attr.is_empty() {
        LitStr::new("", proc_macro2::Span::call_site())
    } else {
        parse_macro_input!(attr as LitStr)
    };
    let path_value = path.value();

    let route_const_name = syn::Ident::new(
        &format!(
            "__ROUTE_{}_{}",
            method,
            func_name.to_string().to_uppercase()
        ),
        func_name.span(),
    );

    let expanded = quote! {
        #(#func_attrs)*
        #func_vis async fn #func_name(#func_inputs) #func_output {
            #func_body
        }

        // Store route metadata as a constant
        pub const #route_const_name: (&'static str, &'static str) = (#method, #path_value);
    };

    TokenStream::from(expanded)
}
