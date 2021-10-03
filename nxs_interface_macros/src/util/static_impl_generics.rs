use std::mem::take;

use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    parse2 as parse, GenericParam, GenericArgument, WhereClause,
    WherePredicate, Lifetime, ImplGenerics, TypeGenerics,
};

use crate::util::AngleBracketList;

/// Prepares generic arguments for a implementing a `'static` trait.
///
/// Takes the output of [`syn::Generics::split_for_impl`] and transforms it into
/// a corresponding triple in which all lifetime parameters in the `impl`
/// parameters and `where` clause have been removed, and all lifetime arguments
/// to the target type have been changed to `'const`.
///
/// This can be useful when writing derive macros for traits such as [`Any`]
/// which require implementing types to have a `'static` lifetime.
pub fn static_impl_generics<'a>(
    (impl_gen, type_gen, where_clause):
    (ImplGenerics<'a>, TypeGenerics<'a>, Option<&'a WhereClause>)
) -> (
    AngleBracketList<GenericParam>,
    AngleBracketList<GenericArgument>,
    Option<WhereClause>,
) {
    static PARSE_ERR: &str = "static_impl_generics: parsing AST tokens failed.";

    // Filter out all lifetimes from the list of generic parameters:
    let mut impl_gen: AngleBracketList<GenericParam>
        = parse(impl_gen.to_token_stream()).expect(PARSE_ERR);
    impl_gen.items = take(&mut impl_gen.items).into_pairs().filter(|pair| {
        match pair.value() {
            GenericParam::Lifetime(_) => false,
            _                         => true,
        }
    }).collect();

    // Replace all lifetimes with 'static in the list of generic arguments:
    let mut type_gen: AngleBracketList<GenericArgument>
        = parse(type_gen.to_token_stream()).expect(PARSE_ERR);
    type_gen.items = take(&mut type_gen.items).into_pairs().map(|mut pair| {
        if let GenericArgument::Lifetime(ref mut lifetime) = pair.value_mut() {
            *lifetime = Lifetime::new("'static", Span::call_site());
        }
        pair
    }).collect();

    // Filter out all type parameters from the `where` clause (if there is one):
    let mut where_clause: Option<WhereClause>
        = where_clause.map(WhereClause::clone);
    if let Some(WhereClause { ref mut predicates, .. }) = &mut where_clause {
        *predicates = take(predicates).into_pairs().filter(|pair| {
            match pair.value() {
                WherePredicate::Lifetime(_) => false,
                _                           => true,
            }
        }).collect();
    }

    (impl_gen, type_gen, where_clause)
}
