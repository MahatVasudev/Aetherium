use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

use crate::utils;

pub fn require_ml(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let vis = &func.vis;
    let sig = &func.sig;
    let block = &func.block;
    let is_async = utils::is_async(&func);

    if !is_async {
        return syn::Error::new_spanned(
            &func.sig.fn_token,
            "require_ml can only be used in async functions",
        )
        .to_compile_error()
        .into();
    }
    let expanded = quote! {
        #vis #sig {
            let __ml_config = match MLConfig::load() {
                Ok(c) => c,
                Err(e) => return EngineResponse::Error {
                    message: e.message().into(),
                },
            };

            let mut ml_client = match MLClient::connect(&__ml_config).await {
                Ok(c) => c,
                Err(_) => return EngineResponse::Error {
                    message: "ML server unavailable. Run 'aetherium ml-server start'".into()
                }
            };
            #block

        }
    };

    TokenStream::from(expanded)
}

pub fn optional_ml(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let vis = &func.vis;
    let sig = &func.sig;
    let block = &func.block;

    let is_async = utils::is_async(&func);

    if !is_async {
        return syn::Error::new_spanned(
            &func.sig.fn_token,
            "optional_ml can only be used in async functions",
        )
        .to_compile_error()
        .into();
    }
    let expanded = quote! {
        #vis #sig {
            let (__ml_available, mut ml_client) = match MLConfig::load(){
                Err(_) => (false, None),
                Ok(__config) => match MLClient::connect(&__config).await {
                    Ok(__client) => (true, Some(__client)),
                    Err(_) => (false, None),
                },
                Err(_) => (false, None),
            };

            #block
        }
    };

    TokenStream::from(expanded)
}
