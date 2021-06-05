#![cfg(test)]

use super::*;
use crate::DynCast;

macro_rules! test_not_cast_borrowed {
    ($value:ident, $cast:ident, $Struct:ident, types($($type:ty,)*)) => {$(
        assert!(!$value.can_cast::<$type>());
        assert!($value.$cast::<$type>().is_none());
    )*}
}

macro_rules! test_cast_borrowed {
    ($value:ident, $cast:ident, $Struct:ident, types($($type:ty,)*)) => {$(
        assert!($value.can_cast::<$type>());
        assert!($value.$cast::<$type>().is_some());
    )*};
}    

macro_rules! test_cast_borrowed_any {
    ($value:ident, $cast:ident, $Struct:ident, types($($type:ty,)*)) => {$(
        assert!($value.can_cast::<$type>());
        assert_eq!($value.$cast::<$type>().map(|a| (*a).type_id()),
                   Some(TypeId::of::<Struct>()));
    )*};
}

macro_rules! test_not_cast_owned {
    ($Ptr:ident, $cast:ident, $Struct:ident, types($($type:ty,)*)) => {$(
        let struct_ptr = $Ptr::new($Struct) as $Ptr<dyn DynCast>;
        assert!(!struct_ptr.can_cast::<$type>());
        assert!(struct_ptr.$cast::<$type>().is_err());
    )*};
}    

macro_rules! test_cast_owned {
    ($Ptr:ident, $cast:ident, $Struct:ident, types($($type:ty,)*)) => {$(
        let struct_ptr = $Ptr::new($Struct) as $Ptr<dyn DynCast>;
        assert!(struct_ptr.can_cast::<$type>());
        assert!(struct_ptr.$cast::<$type>().is_ok());
    )*};
}

macro_rules! test_cast_owned_any {
    ($Ptr:ident, $cast:ident, $Struct:ident, types($($type:ty,)*)) => {$(
        let struct_ptr = $Ptr::new($Struct) as $Ptr<dyn DynCast>;
        assert!(struct_ptr.can_cast::<$type>());
        assert_eq!(struct_ptr.$cast::<$type>().ok().map(|a| (*a).type_id()),
                   Some(TypeId::of::<Struct>()));
    )*};
}

#[test]
fn derive_dyncast_default() {
    //! Deriving `DynCast` with an empty list of base traits and no list of
    //! auto traits specified should leave `Self`, `dyn Any`, `dyn Ant + Send`,
    //! `dyn Any + Sync`, and `dyn Any + Send + Sync` as the only traits
    //! castable to.

    trait Empty {}
    struct Struct;
    DynCast!(Struct, base_traits());

    // cast_ref
    let struct_ref = &Struct as &dyn DynCast;
    test_not_cast_borrowed!(struct_ref, cast_ref, Struct, types(dyn Empty,));
    test_cast_borrowed!(struct_ref, cast_ref, Struct, types(Struct,));
    test_cast_borrowed_any!(struct_ref, cast_ref, Struct, types(
        dyn Any, dyn Any + Send, dyn Any + Sync, dyn Any + Send + Sync,
    ));

    // cast_mut
    let struct_mut = &mut Struct as &mut dyn DynCast;
    test_not_cast_borrowed!(struct_mut, cast_ref, Struct, types(dyn Empty,));
    test_not_cast_borrowed!(struct_mut, cast_mut, Struct, types(dyn Empty,));
    test_cast_borrowed!(struct_mut, cast_mut, Struct, types(Struct,));
    test_cast_borrowed_any!(struct_mut, cast_mut, Struct, types(
        dyn Any, dyn Any + Send, dyn Any + Sync, dyn Any + Send + Sync,
    ));

    // cast_box
    let mut struct_box = Box::new(Struct) as Box<dyn DynCast>;
    test_not_cast_borrowed!(struct_box, cast_ref, Struct, types(dyn Empty,));
    test_not_cast_borrowed!(struct_box, cast_mut, Struct, types(dyn Empty,));
    test_not_cast_owned!(Box, cast_box, Struct, types(dyn Empty,));
    test_cast_owned!(Box, cast_box, Struct, types(Struct,));
    test_cast_owned_any!(Box, cast_box, Struct, types(
        dyn Any, dyn Any + Send, dyn Any + Sync, dyn Any + Send + Sync,
    ));

    // cast_rc
    let struct_rc = Rc::new(Struct) as Rc<dyn DynCast>;
    test_not_cast_borrowed!(struct_rc, cast_ref, Struct, types(dyn Empty,));
    test_not_cast_owned!(Rc, cast_rc, Struct, types(dyn Empty,));
    test_cast_owned!(Rc, cast_rc, Struct, types(Struct,));
    test_cast_owned_any!(Rc, cast_rc, Struct, types(
        dyn Any, dyn Any + Send, dyn Any + Sync, dyn Any + Send + Sync,
    ));

    // cast_arc
    let struct_arc = Arc::new(Struct) as Arc<dyn DynCast>;
    test_not_cast_borrowed!(struct_arc, cast_ref, Struct, types(dyn Empty,));
    test_not_cast_owned!(Arc, cast_arc, Struct, types(dyn Empty,));
    test_cast_owned!(Arc, cast_arc, Struct, types(Struct,));
    test_cast_owned_any!(Arc, cast_arc, Struct, types(
        dyn Any, dyn Any + Send, dyn Any + Sync, dyn Any + Send + Sync,
    ));
}

#[test]
fn derive_dyncast_minimal() {
    //! Deriving `DynCast` with empty lists of base traits and auto traits
    //! should leave `Self` and `dyn Any` as the only types castable to.
    //! In particular, `dyn Any + Send + Sync` should not be castable to, so
    //! that any cast from an `Arc` pointer should fail.

    struct Struct;
    DynCast!(Struct, base_traits(), auto_traits());

    // cast_ref
    let struct_ref = &Struct as &dyn DynCast;
    test_not_cast_borrowed!(struct_ref, cast_ref, Struct, types(
        dyn Any + Send, dyn Any + Sync, dyn Any + Send + Sync,
    ));
    test_cast_borrowed!(struct_ref, cast_ref, Struct, types(Struct,));
    test_cast_borrowed_any!(struct_ref, cast_ref, Struct, types(dyn Any,));

    // cast_arc
    test_not_cast_owned!(Arc, cast_arc, Struct, types(
        dyn Any + Send, dyn Any + Sync, dyn Any + Send + Sync,
    ));

    let struct_arc = Arc::new(Struct) as Arc<dyn DynCast>;
    assert!(struct_arc.can_cast::<Struct>());
    assert!(struct_arc.cast_arc::<Struct>().is_err());

    let struct_arc = Arc::new(Struct) as Arc<dyn DynCast>;
    assert!(struct_arc.can_cast::<dyn Any>());
    assert!(struct_arc.cast_arc::<dyn Any>().is_err());
}
