use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

pub fn cache_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;
    let fn_attrs = &input_fn.attrs;

    // Default configuration (attribute parsing to be added later)
    let key_template = format!("{}:{{:?}}", fn_name);
    let ttl_secs = 3600_u64; // Default 1 hour
    let tags: Vec<String> = Vec::new();

    let cache_code = if tags.is_empty() {
        // Simple cache without tags
        quote! {
            #(#fn_attrs)*
            #fn_vis #fn_sig {
                use std::time::Duration;

                // Generate cache key from function arguments
                let cache_key = format!(#key_template, stringify!(#fn_name));

                // Try to get from cache
                if let Ok(Some(cached)) = __cache.get_json(&cache_key).await {
                    if let Ok(result) = serde_json::from_str(&cached) {
                        return result;
                    }
                }

                // Execute function
                let result = (|| async #fn_block)().await;

                // Cache successful results
                if let Ok(ref success_result) = result {
                    if let Ok(json) = serde_json::to_string(success_result) {
                        let _ = __cache.set_json(
                            &cache_key,
                            json,
                            Some(Duration::from_secs(#ttl_secs))
                        ).await;
                    }
                }

                result
            }
        }
    } else {
        // Tagged cache
        let tags_slice = tags.join("\", \"");
        quote! {
            #(#fn_attrs)*
            #fn_vis #fn_sig {
                use std::time::Duration;

                // Generate cache key from function arguments
                let cache_key = format!(#key_template, stringify!(#fn_name));

                // Try to get from cache
                if let Ok(Some(cached)) = __tagged_cache.get(&cache_key).await {
                    if let Ok(result) = serde_json::from_str(&cached) {
                        return result;
                    }
                }

                // Execute function
                let result = (|| async #fn_block)().await;

                // Cache successful results with tags
                if let Ok(ref success_result) = result {
                    if let Ok(json) = serde_json::to_string(success_result) {
                        let tags: &[&str] = &[#tags_slice];
                        let _ = __tagged_cache.set_with_tags(
                            &cache_key,
                            json,
                            tags,
                            Some(Duration::from_secs(#ttl_secs))
                        ).await;
                    }
                }

                result
            }
        }
    };

    TokenStream::from(cache_code)
}

