//! Automatic derivation of the `DynCast` trait.

use std::collections::HashSet;
use std::iter::FromIterator;
use std::str::FromStr;

use proc_macro2::TokenStream;
use syn::{
    Error, DeriveInput, Path, Attribute, Ident, Meta, NestedMeta, Type,
    MetaList, parse2 as parse, parse_quote as pq, 
};
use quote::{quote as q, ToTokens, TokenStreamExt};
use parse_display::FromStr;

use crate::util::static_impl_generics;

// Representation of *auto traits*, as defined in
// [https://doc.rust-lang.org/reference/special-types-and-traits.html#auto-traits].
//
// This reflects semantic information about the Rust language, so this
// definition (unfortunately) must be updated if the language changes to add
// more auto traits in the future.
#[derive(Clone, Copy, Hash, Eq, PartialEq, FromStr)]
enum AutoTrait {
    Sync, Send, Unpin, UnwindSafe, RefUnwindSafe,
}
impl ToTokens for AutoTrait {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(match self {
            Self::Sync          => q!(::std::marker::Sync),
            Self::Send          => q!(::std::marker::Send),
            Self::Unpin         => q!(::std::marker::Unpin),
            Self::UnwindSafe    => q!(::std::panic::UnwindSafe),
            Self::RefUnwindSafe => q!(::std::panic::RefUnwindSafe),
        })
    }
}

pub fn derive(input: TokenStream) -> syn::Result<TokenStream> {
    #![allow(non_snake_case)]
    let DeriveInput{ attrs, ident, generics, .. } = parse(input)?;
    let (impl_gen, type_gen, where_clause)
        = static_impl_generics(generics.split_for_impl());
    let impl_type = q!(#ident#type_gen);

    // Extract options from helper attributes:
    let mut base_traits: HashSet<Path> = HashSet::new();
    let mut auto_traits: Option<HashSet<AutoTrait>> = None;
    let mut crate_path: Option<Path> = None;
    for attr in attrs {
        read_attr(attr, &mut base_traits, &mut auto_traits, &mut crate_path)?;
    }
    let auto_traits = auto_traits.unwrap_or_else(|| HashSet::from_iter([
        AutoTrait::Send, AutoTrait::Sync
    ]));
    let crate_path = crate_path.unwrap_or_else(|| pq!(::nxs_interface));

    // For later convenience, define the absolute paths of some common items:
    let dyn_cast: Path = pq!(#crate_path::util::dyn_cast);
    let DynCast: Path  = pq!(#dyn_cast::DynCast);
    let Send: Path     = pq!(::std::marker::Send);
    let Sync: Path     = pq!(::std::marker::Sync);
    let Any: Path      = pq!(::std::any::Any);
    let TypeId: Type   = pq!(::std::any::TypeId);
    let Option: Type   = pq!(::std::option::Option);
    let Vec: Type      = pq!(::std::vec::Vec);
    let Box: Type      = pq!(::std::boxed::Box);
    let Rc: Type       = pq!(::std::rc::Rc);
    let Arc: Type      = pq!(::std::sync::Arc);

    // Ensure that `Any` and `DynCast` are among the base traits:
    base_traits.extend([Any.clone(), DynCast.clone()]);

    // Enumerate the types to which casting shall be possible:
    let mut auto_trait_sets = vec![vec![]];
    for auto_trait in &auto_traits {
        let mut sets = auto_trait_sets.clone();
        for set in &mut sets { set.push(auto_trait); }
        auto_trait_sets.append(&mut sets)
    }
    let mut castable = vec![q!(#impl_type)];
    for base_trait in base_traits {
        for auto_traits in &auto_trait_sets {
            castable.push(q!(dyn #base_trait #(+ #auto_traits)* + 'static));
        }
    }

    // Generate the implementation for each method of DynCast:
    macro_rules! cast_meth {(
        // ToTokens; name of this method, e.g. `dyn_cast_ref`.
        $meth_name:expr,

        // Fn(ToTokens, ToTokens) -> ToTokens; transforms a type and a lifetime
        // into a pointer of the kind received by this method, with the given
        // lifetime, if applicable, e.g. `Self` and `'a` into `&'a Self`.
        $ptr_ty:expr,

        // ToTokens; the `downcast` method defined on pointers of the kind
        // received by this method, e.g. `downcast_ref`.
        $dcast_meth:expr,

        // ToTokens; the `dyn Any + ...` type received by the $dcast_meth,
        // e.g. `dyn Any` or `dyn Any + Sync + Send`.
        $dcast_recv:expr,

        // Fn(ToTokens) -> ToTokens; transforms a lifetime into the return type
        // of this method, parameterised by that lifetime if applicable, e.g.
        // `'a` into `DynCastRef<'a>`.
        $res_ty:expr, 
    ) => {{
        let (lt_a, lt_b, lt__) = (q!('a), q!('b), q!('_));
        let meth_name = $meth_name;
        let src_ptr_a = $ptr_ty(&impl_type, &lt_a);
        let res_ty_a = $res_ty(&lt_a);
        let any_ptr_b = $ptr_ty(&$dcast_recv, &lt_b);
        let tgt_ptr_b = castable.iter().map(|t| $ptr_ty(t, &lt_b));
        let tgt_ptr__ = castable.iter().map(|t| $ptr_ty(t, &lt__));
        let dcast_meth = $dcast_meth;
        q!{
            fn #meth_name<#lt_a>(
                self: #src_ptr_a, to: #TypeId
            ) -> #Option<#res_ty_a> {
                #(if to == #TypeId::of::<#castable>() {
                    static CAST:
                        for<#lt_b> fn(#any_ptr_b) -> #Option<#tgt_ptr_b>
                    = |obj| {
                        // To simultaneously handle the cases where #dcast_meth
                        // returns `Option` and, respectively, `Result`, we have
                        // the following awkward but general expression:
                        obj.#dcast_meth::<#impl_type>()
                           .map(|r| #Option::Some(r as #tgt_ptr__))
                           .unwrap_or(#Option::None)
                    };
                    #Option::Some(<#res_ty_a>::from_any_cast_fn(self, &CAST))
                } else)* { #Option::None }
            }
        }
    }}}

    let impl_dyn_cast_methods = [
        cast_meth!(
            q!(dyn_cast_ref), |t, l| q!(&#l (#t)), q!(downcast_ref),
            q!(dyn #Any + 'static), |l| q!(#dyn_cast::DynCastRef<#l>),
        ),
        cast_meth!(
            q!(dyn_cast_mut), |t, l| q!(&#l mut(#t)), q!(downcast_mut),
            q!(dyn #Any + 'static), |l| q!(#dyn_cast::DynCastMut<#l>),
        ),
        cast_meth!(
            q!(dyn_cast_box), |t, _| q!(#Box<#t>), q!(downcast),
            q!(dyn #Any + 'static), |_| q!(#dyn_cast::DynCastBox),
        ),
        cast_meth!(
            q!(dyn_cast_rc), |t, _| q!(#Rc<#t>), q!(downcast),
            q!(dyn #Any + 'static), |_| q!(#dyn_cast::DynCastRc),
        ),
    ];

    let impl_dyn_cast_arc = if auto_traits.contains(&AutoTrait::Sync)
    && auto_traits.contains(&AutoTrait::Send) {
        // If `Send` and `Sync` are present in the list of auto traits,
        // casting between `Arc` pointers is possible, so generate the
        // `dyn_cast_arc` method as normal...
        cast_meth!(
            q!(dyn_cast_arc), |t, _| q!(#Arc<#t>), q!(downcast),
            q!(dyn #Any + #Sync + #Send + 'static),
            |_| q!(#dyn_cast::DynCastArc),
        )
    } else {q!{
        // Otherwise, no such casting is possible, so generate a method
        // that always fails to cast.
        fn dyn_cast_arc(
            self: #Arc<Self>, to: #TypeId
        ) -> #Option<#dyn_cast::DynCastArc> {
            None
        }
    }};

    // Generate the full `impl` statement:
    let output = q!{
        impl#impl_gen #DynCast for #impl_type #where_clause {
            fn dyn_can_cast(&self, to: #TypeId) -> bool {
                [#(#TypeId::of::<#castable>()),*].contains(&to)
            }
            fn castable_types(&self) -> #Vec<#TypeId> {
                vec![#(#TypeId::of::<#castable>()),*]
            }
            #(#impl_dyn_cast_methods)*
            #impl_dyn_cast_arc
        }
    };
    Ok(output)
}

const ATTR_ERR: &str = "Invalid arguments to the `dyn_cast` attribute.";

fn read_attr(
    attr: Attribute,
    base_traits: &mut HashSet<Path>,
    auto_traits: &mut Option<HashSet<AutoTrait>>,
    crate_path: &mut Option<Path>,
) -> syn::Result<()> {
    if !attr.path.is_ident("dyn_cast") { return Ok(()); }
    let list = if let Meta::List(ls) = attr.parse_meta()? { Ok(ls) }
               else { Err(Error::new_spanned(attr, ATTR_ERR)) }?;
    for item in list.nested {
        let meta = if let NestedMeta::Meta(mt) = item { Ok(mt) }
                   else { Err(Error::new_spanned(item, ATTR_ERR)) }?;
        let name = meta.path().get_ident().map(Ident::to_string);
        match (name.as_deref(), meta) {
            (Some("base_traits"), Meta::List(list)) => {
                read_base_traits(list, base_traits)
            }
            (Some("auto_traits"), Meta::List(list)) => {
                read_auto_traits(list, auto_traits)
            }
            (Some("crate"), Meta::List(list)) if list.nested.len() == 1 => {
                read_crate_path(list, crate_path)
            }
            (_, mt) => Err(Error::new_spanned(mt, ATTR_ERR)),
        }?
    }
    Ok(())
}

fn read_base_traits(
    list: MetaList,
    base_traits: &mut HashSet<Path>,
) -> syn::Result<()> {
    for item in list.nested {
        match item {
            NestedMeta::Meta(Meta::Path(path)) => {
                base_traits.insert(path);
            }
            _ => return Err(Error::new_spanned(item, ATTR_ERR)),
        }
    }
    Ok(())
}

fn read_auto_traits(
    list: MetaList,
    auto_traits: &mut Option<HashSet<AutoTrait>>,
) -> syn::Result<()> {
    let auto_traits = auto_traits.get_or_insert_with(HashSet::new);
    for item in list.nested {
        match item {
            NestedMeta::Meta(Meta::Path(path)) => {
                let auto_trait = path.get_ident().map(Ident::to_string)
                    .and_then(|s| AutoTrait::from_str(s.as_str()).ok())
                    .ok_or_else(|| Error::new_spanned(path, ATTR_ERR))?;
                auto_traits.insert(auto_trait);
            }
            _ => return Err(Error::new_spanned(item, ATTR_ERR)),
        }
    }
    Ok(())
}

fn read_crate_path (
    list: MetaList,
    crate_path: &mut Option<Path>,
) -> syn::Result<()> {
    const PATH_ERR: &str = "`path` may not be specified more than once.";
    match (&crate_path, list.nested.into_iter().next()) {
        (None, Some(NestedMeta::Meta(Meta::Path(path)))) => {
            *crate_path = Some(path);
        }
        (None, nm) => return Err(Error::new_spanned(nm, ATTR_ERR)),
        (_,    nm) => return Err(Error::new_spanned(nm, PATH_ERR)),
    }
    Ok(())
}
