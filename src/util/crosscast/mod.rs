use std::any::{Any, TypeId};
use super::dyn_ref::{DynRef, DynMut};

pub mod macros;

// Extends the downcasting behaviour of `Any` with the ability to cast
// `dyn Crosscast` trait objects into any suitable type supported by the
// implementation, possibly including the concrete type `Self` and
// trait objects for combinations of traits implemented on `Self`.
//
// In order to make this trait object-safe, the base methods use
// runtime reflection instead of generic parameters, but the
// `CrosscastExt` extension trait provides the type-safe versions of
// these methods which are intended to be called by users.
pub trait Crosscast: Any {
    // Returns `true` if and only if `self` may be crosscast to
    // the type `T` such that `to = TypeId::of::<T>`.
    fn dyn_may_crosscast(&self, to: TypeId) -> bool {
        self.dyn_crosscast_ref(to).is_some()
    }

    // If `self` crosscasts to `T`, returns some `DynRef` holding
    // a reference to `self` as `T`, where `to = TypeId::of::<T>`;
    // otherwise, returns `None`.
    fn dyn_crosscast_ref(&self, to: TypeId) -> Option<DynRef>;

    // If `self` crosscasts to `T`, returns some `DynMut` holding
    // a mutable reference to `self` as `T`, where `to = TypeId::of::<T>`;
    // otherwise, returns `None`.
    fn dyn_crosscast_mut(&mut self, to: TypeId) -> Option<DynMut>;

    // If `self` crosscasts to `T`, returns some boxed trait object
    // `t: Box<dyn Any>` which downcasts to `Box<Box<T>>`, where the
    // inner box is `self` as `Box<T>` and `to = TypeId::of::<T>`;
    // otherwise, drops `self` and returns None. 
    fn dyn_crosscast_box(self: Box<Self>, to: TypeId) -> Option<Box<dyn Any>>;
}
static CC_PRO_ERR: &str 
    = "The protocol of the `Crosscast` trait has been violated by an instance.";

pub trait CrosscastExt: Crosscast {
    // Returns `true` if and only if `self` may be crosscast to type `T`.
    // Generalises to `Any::is`.
    fn may_crosscast<T: Any + ?Sized>(&self) -> bool;

    // If and only if `self.may_crosscast::<T>()`, returns some immutable
    // reference to `self` as `T`. Generalises `Any::downcast_ref`.
    fn crosscast_ref<T: Any + ?Sized>(&self) -> Option<&T>;

    // If and only if `self.may_crosscast::<T>()`, returns some mutable
    // reference to `self` as `T`. Generalises `Any::downcast_mut`.
    fn crosscast_mut<T: Any + ?Sized>(&mut self) -> Option<&mut T>;
}

impl<S> CrosscastExt for S where S: Crosscast + ?Sized {
    fn may_crosscast<T: Any + ?Sized>(&self) -> bool {
        self.dyn_may_crosscast(TypeId::of::<T>())
    }

    fn crosscast_ref<T: Any + ?Sized>(&self) -> Option<&T> {
        let dr: DynRef = self.dyn_crosscast_ref(TypeId::of::<T>())?;
        Some(dr.downcast_ref::<T>().expect(CC_PRO_ERR))
    }

    fn crosscast_mut<T: Any + ?Sized>(&mut self) -> Option<&mut T> {
        let mut dr: DynMut = self.dyn_crosscast_mut(TypeId::of::<T>())?;
        Some(dr.downcast_mut::<T>().expect(CC_PRO_ERR))
    }
}

// This trait serves to add extension methods to `Box`.
pub trait CrosscastBox: Sized {
    // If `self.may_crosscast::<T>()`, returns some `Box` containing
    // `self` as `T`; otherwise, returns `Err(self)`.
    // Generalises `Box<dyn Any>::downcast`.
    fn crosscast<T: Any + ?Sized>(self) -> Result<Box<T>, Self>;
}

impl<S> CrosscastBox for Box<S> where S: Crosscast + ?Sized {
    fn crosscast<T: Any + ?Sized>(self) -> Result<Box<T>, Self> {
        if !self.may_crosscast::<T>() { return Err(self); }
        let bb = self.dyn_crosscast_box(TypeId::of::<T>()).expect(CC_PRO_ERR);
        Ok(*bb.downcast::<Box<T>>().expect(CC_PRO_ERR))
   }
}
