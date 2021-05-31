#[macro_export]
macro_rules! derive_crosscast {
    // INPUT: when `auto_traits` is not specified, set its default value.
    ($target:ty, base_traits$bs:tt $(,)?) => {
        $crate::derive_crosscast!(
            $target, base_traits$bs,
            auto_traits(::std::marker::Send, ::std::marker::Sync,)
        );
    };

    // INPUT: ensure each list of traits has a trailing comma;
    // add `Any` and `Crosscast` to the list of base traits; initialise the
    // list of sets of auto traits with just the empty set and go to STATE 1.
    ($target:ty,
        base_traits($($b:path),*$(,)?), auto_traits($($a:path),*$(,)*)$(,)?
    ) => {
        $crate::derive_crosscast!(
            @1: $target,
            base_traits(
                ::std::any::Any,
                $crate::util::crosscast::Crosscast,
                $($b,)*
            ),
            auto_traits($($a,)*) -> auto_sets((),)
        );
    };

    // STATE 1: compute all subsets of the given list of auto_traits.
    (@1: $target:ty, base_traits$bs:tt,
        auto_traits($a:path, $($a_:path,)*) -> auto_sets($(($($A:path,)*),)*)
    ) => {
        $crate::derive_crosscast!(
            @1: $target, base_traits$bs,
            auto_traits($($a_,)*) -> auto_sets(
                $(($($A,)*),)*     // all previous sets
                $(($($A,)* $a,),)* // all previous sets, with `a` added to each
            )
        );
    };

    // STATE 1: when finished, initialise the list of castable types with
    // just `Self` and go to STATE 2.
    (@1: $target:ty, base_traits$bs:tt, auto_traits() -> auto_sets$ss:tt) => {
        $crate::derive_crosscast!(
            @2: $target, auto_sets$ss, base_traits$bs -> cast_types(Self,)
        );
    };

    // STATE 2: compute all castable trait object types formed by combining
    // a permissible base trait `b` and a set of permissible auto traits `A`.
    (@2: $target:ty, auto_sets($(($($A:path,)*),)*),
        base_traits($b:path, $($b_:path,)*) -> cast_types($($c:ty,)*)
    ) => {
        $crate::derive_crosscast!(
            @2: $target, auto_sets($(($($A,)*),)*),
            base_traits($($b_,)*) -> cast_types($($c,)* $(dyn $b $(+ $A)*,)*)
        );
    };

    // STATE 2: when finished, generate the final implementation code.
    (@2: $target:ty, auto_sets$_:tt,
        base_traits() -> cast_types($($c:ty,)*)
    ) => {
        impl $crate::util::crosscast::Crosscast for $target {
            fn dyn_may_crosscast(&self, to: ::std::any::TypeId) -> bool {
                let castable = [$(::std::any::TypeId::of::<$c>()),*];
                castable.iter().any(|id| *id == to)
            }
            
            fn dyn_crosscast_ref(&self, to: ::std::any::TypeId)
            -> ::std::option::Option<$crate::util::dyn_ref::DynRef> {
                $(if to == ::std::any::TypeId::of::<$c>() {
                    return ::std::option::Option::Some(
                        $crate::util::dyn_ref::DynRef::new(self as &$c)
                    );
                })*
                ::std::option::Option::None
            }

            fn dyn_crosscast_mut(&mut self, to: ::std::any::TypeId)
            -> ::std::option::Option<$crate::util::dyn_ref::DynMut> {
                $(if to == ::std::any::TypeId::of::<$c>() {
                    return ::std::option::Option::Some(
                        $crate::util::dyn_ref::DynMut::new(self as &mut $c)
                    );
                })*
                ::std::option::Option::None
            }

            fn dyn_crosscast_box(
                self: ::std::boxed::Box<Self>, to: ::std::any::TypeId
            ) -> ::std::option::Option<::std::boxed::Box<dyn ::std::any::Any>> {
                $(if to == ::std::any::TypeId::of::<$c>() {
                    return ::std::option::Option::Some(
                        ::std::boxed::Box::new(self as ::std::boxed::Box<$c>)
                    );
                })*
                ::std::option::Option::None
            }
        }
    };
}
