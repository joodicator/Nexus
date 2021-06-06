//! Exports the `DynCast` trait and related items.

use std::any::{Any, TypeId};
use std::{rc::Rc, sync::Arc};
use std::marker::{Sync, Send};

mod macros;
mod tests;

/// Trait providing a generalised form of dynamic typing.
///
/// Extends the downcasting behaviour of `Any` with the ability to cast into
/// any arbitrary type supported by the implementation, based on dynamic runtime
/// type reflection.
///
/// The motivating use case is *cross-casting* between different trait object
/// representations of the same value, e.g. casting from `&dyn DynCast` to `&dyn R`
/// for some trait `R`, and, if `R: DynCast` and the implementation allows it,
/// casting back from `&dyn R` to `&dyn DynCast`.
/// 
/// For users, the extension trait `DynCastExt` should be imported alongside
/// `DynCast` to enable the intended high-level user interface. The methods in
/// this trait have unusual types due to the constraints of object-safety, making
/// them inconvenient for general use.
///
/// For implementors, the macro `DynCast` can be used to derive an implementation
/// that allows casting into any trait object from a finite set of declared traits.
pub trait DynCast: Any {
    /// Tells whether casting to a given `TypeId` is possible.
    ///
    /// Specifically, if `*self` can be cast to the type `T` for which
    /// `TypeId::of::<T> == to`, returns `true`, or otherwise `false`.
    fn dyn_can_cast(&self, to: TypeId) -> bool {
        self.dyn_cast_ref(to).is_some()
    }

    /// Attempts to cast a shared reference to a given `TypeId`.
    ///
    /// If `*self` can be cast to the type `T` for which `TypeId::of::<T> == to`,
    /// returns some `DynCastRef` yielding `self` as `&T`, or else `None`.
    fn dyn_cast_ref(&self, to: TypeId) -> Option<DynCastRef>;

    /// Attempts to cast a mutable reference to a given `TypeId`.
    ///
    /// If `*self` can be cast to the type `T` for which `TypeId::of::<T> == to`,
    /// returns some `DynCastMut` yielding `self` as `&mut T`, or else `None`.
    fn dyn_cast_mut(&mut self, to: TypeId) -> Option<DynCastMut>;

    /// Attempts to cast a box to a given `TypeId`.
    ///
    /// If `*self` can be cast to the type `T` for which `TypeId::of::<T> == to`,
    /// returns some `DynCastBox` yielding `self` as `Box<T>`, or else `None`.
    fn dyn_cast_box(self: Box<Self>, to: TypeId) -> Option<DynCastBox>;

    /// Attempts to cast a reference-counted pointer to a given `TypeId`.
    ///
    /// If `*self` can be cast to the type `T` for which `TypeId::of::<T> == to`,
    /// returns some `DynCastRc` yielding `self` as `Rc<T>`, or else `None`.
    fn dyn_cast_rc(self: Rc<Self>, to: TypeId) -> Option<DynCastRc>;

    /// Attempts to cast an atomically reference-counted pointer to a given
    /// `TypeId`.
    ///
    /// If `*self` can be cast to the type `T` for which `TypeId::of::<T> == to`
    /// **and** `*self` can be cast to `dyn Any + Sync + Send`, returns some
    /// `DynCastArc` yielding `self` as `Arc<T>`, or else `None`.
    fn dyn_cast_arc(self: Arc<Self>, to: TypeId) -> Option<DynCastArc>;
}

/// The successful return type of `DynCast::dyn_cast_ref`.
pub struct DynCastRef<'a> { src: &'a dyn Any, fun: &'static dyn Any }
impl<'a> DynCastRef<'a> {
    /// Constructs a `DynCastRef` from a reference that can be upcast to
    /// `&dyn Any` and a function that can downcast it thence to `&T`.
    pub fn from_downcast_fn<S: Any, T: Any + ?Sized>(
        src: &'a S, fun: &'static fn(&dyn Any) -> Option<&T>,
    ) -> Self { Self { src, fun } }

    /// Returns the casted reference, given its correct referent type.
    pub fn cast<T: Any + ?Sized>(self) -> Option<&'a T> {
        self.fun.downcast_ref::<fn(&dyn Any) -> Option<&T>>()?(self.src)
    }
}

/// The successful return type of `DynCast::dyn_cast_mut`.
pub struct DynCastMut<'a> { src: &'a mut dyn Any, fun: &'static dyn Any }
impl<'a> DynCastMut<'a> {
    /// Constructs a `DynCastMut` from a reference that can be upcast to
    /// `&mut dyn Any` and a function that can downcast it thence to `&mut T`.
    pub fn from_downcast_fn<S: Any, T: Any + ?Sized>(
        src: &'a mut S, fun: &'static fn(&mut dyn Any) -> Option<&mut T>,
    ) -> Self { Self { src, fun } }

    /// Returns the casted reference, given its correct referent type.
    pub fn cast<T: Any + ?Sized>(self) -> Option<&'a mut T> {
        self.fun.downcast_ref::<fn(&mut dyn Any) -> Option<&mut T>>()?(self.src)
    }
}

/// The successful return type of `DynCast::dyn_cast_box`.
pub struct DynCastBox { src: Box<dyn Any>, fun: &'static dyn Any }
impl<'a> DynCastBox {
    /// Constructs a `DynCastBox` from a box that can be upcast to
    /// `Box<dyn Any>` and a function that can downcast it thence to `Box<T>`.
    pub fn from_downcast_fn<S: Any, T: Any + ?Sized>(
        src: Box<S>, fun: &'static fn(Box<dyn Any>) -> Option<Box<T>>,
    ) -> Self { Self { src, fun } }

    /// Returns the casted box, given its correct referent type.
    pub fn cast<T: Any + ?Sized>(self) -> Option<Box<T>> {
        self.fun.downcast_ref::<fn(Box<dyn Any>) -> Option<Box<T>>>()?(self.src)
    }
}

/// The successful return type of `DynCast::dyn_cast_rc`.
pub struct DynCastRc { src: Rc<dyn Any>, fun: &'static dyn Any }
impl<'a> DynCastRc {
    /// Constructs a `DynCastRc` from a pointer that can be upcast to
    /// `Rc<dyn Any>` and a function that can downcast it thence to `Rc<T>`.
    pub fn from_downcast_fn<S: Any, T: Any + ?Sized>(
        src: Rc<S>, fun: &'static fn(Rc<dyn Any>) -> Option<Rc<T>>,
    ) -> Self { Self { src, fun } }

    /// Returns the casted pointer, given its correct referent type.
    pub fn cast<T: Any + ?Sized>(self) -> Option<Rc<T>> {
        self.fun.downcast_ref::<fn(Rc<dyn Any>) -> Option<Rc<T>>>()?(self.src)
    }
}

/// The successful return type of `DynCast::dyn_cast_arc`.
pub struct DynCastArc { src: Arc<dyn Any + Send + Sync>, fun: &'static dyn Any }
impl<'a> DynCastArc {
    /// Constructs a `DynCastArc` from a pointer that can be upcast to
    /// `Arc<dyn Any + Send + Sync>` and a function that can downcast it thence
    /// to `Arc<T + Send Sync>`.
    pub fn from_downcast_fn<S: Any + Send + Sync, T: Any + ?Sized>(
        src: Arc<S>,
        fun: &'static fn(Arc<dyn Any + Send + Sync>) -> Option<Arc<T>>,
    ) -> Self { Self { src, fun } }

    /// Returns the casted pointer, given its correct referent type.
    pub fn cast<T: Any + ?Sized>(self) -> Option<Arc<T>> {
        self.fun.downcast_ref::<fn(Arc<dyn Any + Send + Sync>)
                                -> Option<Arc<T>>>()?(self.src)
    }
}

const DYNCAST_ERR: &str
    = "The contract of `DynCast` has been broken by an implementation.";

/// User-friendly extension methods for `DynCast`.
/// 
/// This extension trait contains non-object-safe generic methods necessary for
/// users to interact with the `DynCast` trait in a type-safe manner. A blanket
/// implementation makes it available on all `dyn DynCast` trait objects.
pub trait DynCastExt: DynCast {
    /// Tells whether casting to a given type argument is possible.
    ///
    /// Specifically, returns `true` if `self` can be cast to type `T`, or
    /// otherwise `false`.
    fn can_cast<T: Any + ?Sized>(&self) -> bool {
        self.dyn_can_cast(TypeId::of::<T>())
    }

    /// Attempts to cast a shared reference to a given type.
    /// 
    /// If `self` can be cast to type `T`, returns `Some` with the given shared
    /// reference cast from `&Self` to `&T`, or otherwise `None`.
    fn cast_ref<T: Any + ?Sized>(&self) -> Option<&T> where {
        let res = self.dyn_cast_ref(TypeId::of::<T>())?;
        Some(res.cast::<T>().expect(DYNCAST_ERR))
    }

    /// Attempts to cast a mutable reference to a given type.
    ///
    /// If `self` can be cast to type `T`, returns `Some` with the given mutable
    /// reference cast from `&mut Self` to `&mut T`, or otherwise `None`.
    fn cast_mut<T: Any + ?Sized>(&mut self) -> Option<&mut T> {
        let res = self.dyn_cast_mut(TypeId::of::<T>())?;
        Some(res.cast::<T>().expect(DYNCAST_ERR))
    }

    /// Attempts to cast a box to a given type.
    ///
    /// If `self` can be cast to type `T`, returns `Ok` with the given box cast
    /// from `Box<Self>` to `Box<T>`, or otherwise `Err` with the original box.
    fn cast_box<T: Any + ?Sized>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if !self.can_cast::<T>() { return Err(self); }
        let res = self.dyn_cast_box(TypeId::of::<T>()).expect(DYNCAST_ERR);
        Ok(res.cast::<T>().expect(DYNCAST_ERR))
    }

    /// Attempts to cast a reference-counted pointer to a given type.
    ///
    /// If `self` can be cast to type `T`, returns `Ok` with the given
    /// reference-counted pointer cast from `Rc<Self>` to `Rc<T>`, or otherwise
    /// `Err` with the original pointer.
    fn cast_rc<T: Any + ?Sized>(self: Rc<Self>) -> Result<Rc<T>, Rc<Self>> {
        if !self.can_cast::<T>() { return Err(self); }
        let res = self.dyn_cast_rc(TypeId::of::<T>()).expect(DYNCAST_ERR);
        Ok(res.cast::<T>().expect(DYNCAST_ERR))
    }

    /// Attempts to cast an atomically reference-counted pointer to a given
    /// type.
    ///
    /// If `self` can be cast to the types `T` **and** `dyn Any + Sync + Send`,
    /// returns `Ok` with the given atomically reference-counted pointer cast
    /// from `Arc<Self>` to `Arc<T>`, or otherwise `Err` with the original
    /// pointer.
    fn cast_arc<T: Any + ?Sized>(self: Arc<Self>) -> Result<Arc<T>, Arc<Self>> {
        if !self.can_cast::<T>() { return Err(self); }
        if !self.can_cast::<dyn Any + Send + Sync>() { return Err(self); }
        let res = self.dyn_cast_arc(TypeId::of::<T>()).expect(DYNCAST_ERR);
        Ok(res.cast::<T>().expect(DYNCAST_ERR))
    }
}
impl<S> DynCastExt for S where S: DynCast + ?Sized {}
