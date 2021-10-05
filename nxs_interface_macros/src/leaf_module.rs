use proc_macro2::TokenStream;
use syn::{
    Error, DeriveInput, Path, Type, Meta, NestedMeta, Ident,
    parse2 as parse, parse_quote as pq
};
use quote::quote as q;

use crate::util::static_impl_generics;

pub fn derive(input: TokenStream) -> syn::Result<TokenStream> {
    #![allow(non_snake_case)]

    // Parse raw input, asserting that the target type is `'static`:
    let DeriveInput{ attrs, ident, generics, .. } = parse(input)?;
    let (impl_gen, type_gen, where_clause)
        = static_impl_generics(generics.split_for_impl());

    // Extract options from helper attributes:
    let mut crate_path: Option<Path> = None;
    const ATTR_ERR: &str = "Invalid argument(s) to `leaf_module` attribute.";
    const PATH_ERR: &str = "`path` may not be specified more than once.";
    for attr in attrs {
        if !attr.path.is_ident("leaf_module") { continue; }
        let list = if let Meta::List(ls) = attr.parse_meta()? { Ok(ls) }
                   else { Err(Error::new_spanned(attr, ATTR_ERR)) }?;
        for item in list.nested {
            let meta = if let NestedMeta::Meta(m) = item { Ok(m) }
                       else { Err(Error::new_spanned(item, ATTR_ERR)) }?;
            let name = meta.path().get_ident().map(Ident::to_string);
            match (name.as_deref(), meta) {
                (Some("crate"), Meta::List(list)) if list.nested.len() == 1 => {
                    match (&crate_path, list.nested.into_iter().next()) {
                        (None, Some(NestedMeta::Meta(Meta::Path(path)))) => {
                            crate_path = Some(path); Ok(())
                        }
                        (None, nm) => Err(Error::new_spanned(nm, ATTR_ERR)),
                        (_,    nm) => Err(Error::new_spanned(nm, PATH_ERR)),
                    }
                }
                (_, mt) => Err(Error::new_spanned(mt, ATTR_ERR)),
            }?
        }
    }
    let crate_path = crate_path.unwrap_or_else(|| pq!(::nxs_interface));

    // Define paths and types for quote interpolation:
    let LeafModule: Path = pq!(#crate_path::root::LeafModule);
    let RootModule: Path = pq!(#crate_path::root::RootModule);
    let Pin: Type        = pq!(::std::pin::Pin);
    let Box: Type        = pq!(::std::boxed::Box);
    let Future: Path     = pq!(::std::future::Future);
    let Send: Path       = pq!(::std::marker::Send);

    let impl_type = q!(#ident#type_gen);
    let BoxFuture = |a, T| q!(#Pin<#Box<dyn #Future<Output = #T> + #Send + #a>>);
    let result = q!(#crate_path::Result<#Box<dyn #LeafModule + 'static>>);
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
