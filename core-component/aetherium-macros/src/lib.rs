mod engine_middleware;
mod utils;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn require_ml(_attr: TokenStream, item: TokenStream) -> TokenStream {
    engine_middleware::require_ml(_attr, item)
}

#[proc_macro_attribute]
pub fn optional_ml(_attr: TokenStream, item: TokenStream) -> TokenStream {
    engine_middleware::optional_ml(_attr, item)
}
