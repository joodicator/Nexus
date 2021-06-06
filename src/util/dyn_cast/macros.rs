//! Macros associated with `dyn_cast` that might be used in other modules.

/// Derives an instance of the [`DynCast`] trait.
///
/// # Usage
/// ```text
/// DynCast!(ImplType, base_traits(B1, B2, ..., Bm), auto_traits(A1, A2, ..., An));
/// ```
/// or
/// ```text
/// DynCast!(ImplType, base_traits(B1, B2, ..., Bm));
/// ```
/// where:
/// * `ImplType` is a type, usually the name of a `struct` or `enum`.
/// * Each `Bi` is a trait implemented by `ImplType` usable as the *base trait*
///   of a [trait object], **except** for [`DynCast`] or [`Any`].
/// * Each `Aj` is an [auto trait] implemented by `ImplType`. Auto traits must
///   be specified by one of the identifiers `Send`, `Sync`, `Unpin`,
///   `UnwindSafe`, or `RefUnwindsafe`: paths or type aliases are not accepted,
///   and these identifiers refer to the standard traits they name regardless
///   of what is in scope at the call site.
///   
///   If the `auto_traits` key is not specified, it defaults to
///   `auto_traits(Send, Sync)`.
///
/// An invocation of this macro in Item position subject to the above will attempt
/// to generate an implementation `impl DynCast for ImplType { ... }` declaring
/// `ImplType` to be *castable to* exactly the following types:
/// * `ImplType` itself.
/// * Any trait object formed by combining:
///   * exactly one of `Any`, `DynCast` or one of the given base traits `Bi`, with
///   * zero or more of the given (or chosen by default) auto traits `Aj`.
///
/// # Examples
/// ## 1. Possible combinations of base and auto traits
/// ```
/// # use std::{marker::Unpin, any::Any};
/// use untitled::util::dyn_cast::{DynCast, DynCastExt};
/// use untitled::DynCast; // this macro
/// 
/// struct Struct1 { /* ... */ }
/// trait Trait1 { /* ... */ }
/// trait Trait2 { /* ... */ }
/// impl Trait1 for Struct1 { /* ... */ }
/// DynCast!(Struct1, base_traits(Trait1), auto_traits(Send, Unpin));
///
/// let obj = &(Struct1 {/* ... */}) as &dyn DynCast;
///
/// // `obj` can be cast to the following 13 types:
/// assert!(obj.can_cast::<Struct1>());
/// assert!(obj.can_cast::<dyn Trait1>());
/// assert!(obj.can_cast::<dyn Trait1 + Send>());
/// assert!(obj.can_cast::<dyn Trait1 + Unpin>());
/// assert!(obj.can_cast::<dyn Trait1 + Send + Unpin>());
/// assert!(obj.can_cast::<dyn DynCast>());
/// assert!(obj.can_cast::<dyn DynCast + Send>());
/// assert!(obj.can_cast::<dyn DynCast + Unpin>());
/// assert!(obj.can_cast::<dyn DynCast + Send + Unpin>());
/// assert!(obj.can_cast::<dyn Any>());
/// assert!(obj.can_cast::<dyn Any + Send>());
/// assert!(obj.can_cast::<dyn Any + Unpin>());
/// assert!(obj.can_cast::<dyn Any + Send + Unpin>());
///
/// // But not, for example, this type, which was absent from the declaration:
/// assert!(!obj.can_cast::<dyn Trait2>());
/// ```
///
/// ## 2. Declaring many base traits
/// ```
/// # use untitled::util::dyn_cast::{DynCast, DynCastExt};
/// # use untitled::DynCast; // this macro
///
/// struct Struct2 { /* ... */ }
/// # trait Trait1 { /* ... */ }
/// # trait Trait2 { /* ... */ }
/// trait Trait3 { /* ... */ }
/// trait Trait4 { /* ... */ }
/// impl Trait1 for Struct2 { /* ... */ }
/// impl Trait2 for Struct2 { /* ... */ }
/// impl Trait3 for Struct2 { /* ... */ }
/// impl Trait4 for Struct2 { /* ... */ }
/// DynCast!(Struct2, base_traits(Trait1, Trait2, Trait3, Trait4));
///
/// let obj = &(Struct2 { /* ... */ }) as &dyn DynCast;
///
/// // `obj` can be cast to any of these types...
/// assert!(obj.can_cast::<dyn Trait1>());
/// assert!(obj.can_cast::<dyn Trait2>());
/// assert!(obj.can_cast::<dyn Trait3>());
/// assert!(obj.can_cast::<dyn Trait4>());
/// // ...among others.
/// ```
/// [trait object]: https://doc.rust-lang.org/reference/types/trait-object.html
/// [auto trait]: https://doc.rust-lang.org/reference/special-types-and-traits.html#auto-traits
/// [`Any`]: std::any::Any
#[macro_export]
macro_rules! DynCast {
    // INPUT: when `auto_traits` is not specified, set its default value.
    ($target:ty, base_traits$bs:tt $(,)?) => {
        $crate::DynCast!{$target, base_traits$bs, auto_traits(Send, Sync)}
    };
    
    // INPUT: ensure each list of traits has a trailing comma; add `Any` and
    // `DynCast` to the list of base traits; initialise some flags for later use
    // and initialise the list of canonical auto traits as empty and go to STATE 1.
    (   $target:ty,
        base_traits($($b:path),*$(,)?), auto_traits($($a:tt),*$(,)*)$(,)?
    ) => {$crate::DynCast!{
        @1: $target, flags(Send=false, Sync=false),
        base_traits(::std::any::Any, $crate::util::dyn_cast::DynCast, $($b,)*),
        auto_traits($($a,)*) -> ()
    }};

    // STATE 1: canonicalise all auto trait paths and record flags for whether
    // the particular traits Send and Sync are present in the list.
    (   @1: $target:ty, flags(Send=$_:tt, Sync=$sync:tt), base_traits$bs:tt,
        auto_traits(Send, $($ai:tt,)*) -> ($($ao:path,)*)
    ) => {$crate::DynCast!{
        @1: $target, flags(Send=true, Sync=$sync), base_traits$bs,
        auto_traits($($ai,)*) -> ($($ao,)* ::std::marker::Send,)
    }};
    (   @1: $target:ty, flags(Send=$send:tt, Sync=$_:tt), base_traits$bs:tt,
        auto_traits(Sync, $($ai:tt,)*) -> ($($ao:path,)*)
    ) => {$crate::DynCast!{
        @1: $target, flags(Send=$send, Sync=true), base_traits$bs,
        auto_traits($($ai,)*) -> ($($ao,)* ::std::marker::Sync,)
    }};
    (   @1: $target:ty, flags$f:tt, base_traits$bs:tt,
        auto_traits(Unpin, $($ai:tt,)*) -> ($($ao:path,)*)
    ) => {$crate::DynCast!{
        @1: $target, flags$f, base_traits$bs,
        auto_traits($($ai,)*) -> ($($ao,)* ::std::marker::Unpin,)
    }};
    (   @1: $target:ty, flags$f:tt, base_traits$bs:tt,
        auto_traits(UnwindSafe, $($ai:tt,)*) -> ($($ao:path,)*)
    ) => {$crate::DynCast!{
        @1: $target, flags$f, base_traits$bs,
        auto_traits($($ai,)*) -> ($($ao,)* ::std::panic::UnwindSafe,)
    }};
    (   @1: $target:ty, flags$f:tt, base_traits$bs:tt,
        auto_traits(RefUnwindSafe, $($ai:tt,)*) -> ($($ao:path,)*)
    ) => {$crate::DynCast!{
        @1: $target, flags$f, base_traits$bs,
        auto_traits($($ai,)*) -> ($($ao,)* ::std::panic::RefUnwindSafe,)
    }};

    // STATE 1: when finished, initialise the list of sets of auto traits with
    // just the empty set and go to STATE 2.
    (   @1: $target:ty, flags$f:tt, base_traits$bs:tt, auto_traits() -> $as:tt
    ) => {$crate::DynCast!{
        @2: $target, flags$f, base_traits$bs, auto_traits$as -> auto_sets((),)
    }};

    // STATE 2: compute all subsets of the given list of auto_traits.
    (   @2: $target:ty, flags$f:tt, base_traits$bs:tt,
        auto_traits($a:path, $($a_:path,)*) -> auto_sets($(($($A:path,)*),)*)
    ) => {$crate::DynCast!{ 
        @2: $target, flags$f, base_traits$bs,
        auto_traits($($a_,)*) -> auto_sets($(($($A,)*),)* $(($($A,)* $a,),)*)
    }};

    // STATE 2: when finished, initialise the list of castable types with
    // just the concrete base type (`$target`) and go to STATE 3.
    (   @2: $target:ty, flags$f:tt, base_traits$bs:tt,
        auto_traits() -> auto_sets$ss:tt
    ) => {$crate::DynCast!{
        @3: $target, flags$f, auto_sets$ss,
        base_traits$bs -> cast_types($target,)
    }};

    // STATE 3: compute all castable trait object types formed by combining
    // a permissible base trait `b` and a set of permissible auto traits `A`.
    (   @3: $target:ty, flags$f:tt, auto_sets($(($($A:path,)*),)*),
        base_traits($b:path, $($b_:path,)*) -> cast_types($($c:ty,)*)
    ) => {$crate::DynCast!{
        @3: $target, flags$f, auto_sets($(($($A,)*),)*), base_traits($($b_,)*)
        -> cast_types($($c,)* $(dyn $b $(+ $A)* + 'static,)*)
    }};

    // STATE 3, OUTPUT: when finished, generate the code for the implementation.
    (   @3: $target:ty, flags$f:tt, auto_sets$_:tt,
        base_traits() -> cast_types($($c:ty,)*)
    ) => {impl $crate::util::dyn_cast::DynCast for $target {
        #![allow(unused_parens, unused_variables)]

        fn dyn_can_cast(&self, to: ::std::any::TypeId) -> bool {
            $(to == ::std::any::TypeId::of::<$c>() ||)* false
        }
        $crate::DynCast!(
            @5: dyn_cast_ref, $target, cast_types($($c,)*),
            p_cast_types($(&($c),)*),
            misc(p_target = &($target),
                 p_any    = &dyn ::std::any::Any,
                 downcast = downcast_ref,
                 result = $crate::util::dyn_cast::DynCastRef)
        );
        $crate::DynCast!(
            @5: dyn_cast_mut, $target, cast_types($($c,)*),
            p_cast_types($(&mut($c),)*),
            misc(p_target = &mut($target),
                 p_any    = &mut dyn ::std::any::Any,
                 downcast = downcast_mut,
                 result = $crate::util::dyn_cast::DynCastMut)
        );
        $crate::DynCast!(
            @5: dyn_cast_box, $target, cast_types($($c,)*),
            p_cast_types($(::std::boxed::Box<$c>,)*),
            misc(p_target = ::std::boxed::Box<$target>,
                 p_any    = ::std::boxed::Box<dyn ::std::any::Any>,
                 downcast = downcast,
                 result = $crate::util::dyn_cast::DynCastBox)
        );
        $crate::DynCast!(
            @5: dyn_cast_rc, $target, cast_types($($c,)*),
            p_cast_types($(::std::rc::Rc<$c>,)*),
            misc(p_target = ::std::rc::Rc<$target>,
                 p_any    = ::std::rc::Rc<dyn ::std::any::Any>,
                 downcast = downcast,
                 result = $crate::util::dyn_cast::DynCastRc)
        );
        $crate::DynCast!(
            @4: dyn_cast_arc, $target, flags$f, cast_types($($c,)*),
            p_cast_types($(::std::sync::Arc<$c>,)*),
            misc(p_target = ::std::sync::Arc<$target>,
                 p_any    = ::std::sync::Arc<dyn ::std::any::Any +
                                ::std::marker::Sync + ::std::marker::Send>,
                 downcast = downcast,
                 result = $crate::util::dyn_cast::DynCastArc)
        );
    }};

    // STATE 4: generate the normal code for `dyn_cast_arc`, provided that the
    // `Send` and `Sync` traits are both supported by the implementation.
    (   @4: dyn_cast_arc, $target:ty, flags(Send=true, Sync=true),
        cast_types$cs:tt, p_cast_types$ps:tt, misc$ms:tt
    ) => {$crate::DynCast!{
        @5: dyn_cast_arc, $target, cast_types$cs, p_cast_types$ps, misc$ms
    }};

    // STATE 4: otherwise, generate an implementation of `dyn_cast_arc` that
    // refuses to cast to any type.
    (@4: dyn_cast_arc, $target:ty, flags$_f:tt,
        cast_types$_cs:tt, p_cast_types$_ps:tt, misc$ms:tt
    ) => {
        $crate::DynCast!(@5: dyn_cast_arc, $target,
            cast_types(), p_cast_types(), misc$ms
        );
    };

    // STATE 5: generate the code for an individual casting method.
    (   @5: $method:ident, $target:ty,
        cast_types($($to:ty,)*), p_cast_types($($p_to:ty,)*),
        misc(p_target=$p_target:ty, p_any=$p_any:ty, downcast=$downcast:ident,
             result=$result:ty)
    ) => {
        fn $method(self: $p_target, to: ::std::any::TypeId)
        -> ::std::option::Option<$result> {
            $(if to == ::std::any::TypeId::of::<$to>() {
                fn downcast(self_any: $p_any) -> Option<$p_to> {
                    self_any.$downcast::<$target>().map(
                        |t| ::std::option::Option::Some(t as $p_to)
                    ).unwrap_or(::std::option::Option::None)
                }
                static DOWNCAST_FP: fn($p_any) -> Option<$p_to> = downcast;
                Some(<$result>::from_downcast_fn(self, &DOWNCAST_FP))
            } else)* {
                None
            }
        }
    };
}
