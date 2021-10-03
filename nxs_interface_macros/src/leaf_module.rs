use proc_macro2::TokenStream;
use syn::{DeriveInput, Path, Type, parse2 as parse, parse_quote as pq};
use quote::quote as q;

use crate::util::static_impl_generics;

pub fn derive(input: TokenStream) -> syn::Result<TokenStream> {
    #![allow(non_snake_case)]

    let DeriveInput{ ident, generics, .. } = parse(input)?;
    let (impl_gen, type_gen, where_clause)
        = static_impl_generics(generics.split_for_impl());

    let nxs: Path        = pq!(::nxs_interface);
    let LeafModule: Path = pq!(#nxs::root::LeafModule);
    let RootModule: Path = pq!(#nxs::root::RootModule);
    let Pin: Type        = pq!(::std::pin::Pin);
    let Box: Type        = pq!(::std::boxed::Box);
    let Future: Path     = pq!(::std::future::Future);
    let Send: Path       = pq!(::std::marker::Send);

    let impl_type = q!(#ident#type_gen);

    let BoxFuture = |a, T| q!(#Pin<#Box<dyn #Future<Output = #T> + #Send + #a>>);
    let result = q!(#nxs::Result<#Box<dyn #LeafModule + 'static>>);
    let result = BoxFuture(q!('static), result);

    Ok(q!{
        impl#impl_gen #LeafModule for #impl_type #where_clause {
            fn dyn_load(root: &'static (dyn #RootModule + 'static))
            -> #result where Self: Sized {
                #Box::pin(async move {Ok(
                    #Box::new(<#impl_type>::load(root).await?)
                    as #Box<dyn #LeafModule>
                )})
            }
        }
    })
}
