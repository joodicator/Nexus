//! Procedural macros for `nxs_interface`.

use proc_macro::TokenStream;

mod util;


mod dyn_cast;

#[proc_macro_derive(DynCast, attributes(dyn_cast))]
pub fn derive_dyn_cast(input: TokenStream) -> TokenStream {
    dyn_cast::derive(input.into()).unwrap_or_else(
        |e| e.into_compile_error().into()
    ).into()
}


mod leaf_module;

#[proc_macro_derive(LeafModule)]
pub fn derive_leaf_module(input: TokenStream) -> TokenStream {
    leaf_module::derive(input.into()).unwrap_or_else(
        |e| e.into_compile_error().into()
    ).into()
}
