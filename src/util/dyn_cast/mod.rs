use std::any::{Any, TypeId};
use std::{rc::Rc, sync::Arc};
use std::marker::{Sync, Send};

pub mod macros;
mod tests;

// Extends the downcasting behaviour of `Any` with the ability to cast into
// any arbitrary type supported by the implementation, based on dynamic runtime
// type reflection.
//
// The motivating use case is *cross-casting* between different trait object
// representations of the same value, e.g. casting from `&dyn DynCast` to `&dyn R`
// for some trait `R`, and, if `R: DynCast` and the implementation allows it,
// casting back from `&dyn R` to `&dyn DynCast`.
// 
// For users, the extension trait `DynCastExt` should be imported alongside
// `DynCast` to enable the intended high-level user interface. The methods in
// this trait have unusual types due to the constraints of object-safety, making
// them inconvenient for general use.
//
// For implementors, the macro `DynCast` can be used to derive an implementation
// that allows casting into any trait object from a finite set of declared traits.
pub trait DynCast: Any {
    // If `*self` can be cast to the type `T` for which `TypeId::of::<T> == to`,
    // returns `true`, or otherwise `false`.
    fn dyn_can_cast(&self, to: TypeId) -> bool {
        self.dyn_cast_ref(to).is_some()
    }

    // If `*self` can be cast to the type `T` for which `TypeId::of::<T> == to`,
    // returns some `DynCastRef` yielding `self` as `&T`, or else `None`.
    fn dyn_cast_ref(&self, to: TypeId) -> Option<DynCastRef>;

    // If `*self` can be cast to the type `T` for which `TypeId::of::<T> == to`,
    // returns some `DynCastMut` yielding `self` as `&mut T`, or else `None`.
    fn dyn_cast_mut(&mut self, to: TypeId) -> Option<DynCastMut>;

    // If `*self` can be cast to the type `T` for which `TypeId::of::<T> == to`,
    // returns some `DynCastBox` yielding `self` as `Box<T>`, or else `None`.
    fn dyn_cast_box(self: Box<Self>, to: TypeId) -> Option<DynCastBox>;

    // If `*self` can be cast to the type `T` for which `TypeId::of::<T> == to`,
    // returns some `DynCastRc` yielding `self` as `Rc<T>`, or else `None`.
    fn dyn_cast_rc(self: Rc<Self>, to: TypeId) -> Option<DynCastRc>;

    // If `*self` can be cast to the type `T` for which `TypeId::of::<T> == to`
    // **and** `*self` can be cast to `dyn Any + Sync + Send`, returns some
    // `DynCastArc` yielding `self` as `Arc<T>`, or else `None`.
    fn dyn_cast_arc(self: Arc<Self>, to: TypeId) -> Option<DynCastArc>;
}

// A dynamically-typed shared reference ready to be cast to the type `&T`
// for some `T: 'static + ?Sized`.
pub struct DynCastRef<'a> { src: &'a dyn Any, fun: &'static dyn Any }

impl<'a> DynCastRef<'a> {
    // Constructs a `DynCastRef` that may be cast to `&T`, given an underlying
    // shared reference and a function which, given the underlying reference as
    // `&dyn Any`, performs the casting to `&T`.
    //
    // Note that, for technical reasons, `fun` must be given indirectly as a
    // static reference to a function pointer to the actual function. 
    pub fn from_downcast_fn<S: Any, T: ?Sized + 'static>(
        src: &'a S, fun: &'static fn(&dyn Any) -> Option<&T>
    ) -> Self {
        Self { src, fun }
    }

    // Consumes the `DynCastRef`, yielding some casting result of type `&T`,
    // if this is the correct type, or else `None`.
    pub fn cast<T: 'static + ?Sized>(self) -> Option<&'a T> {
        self.fun.downcast_ref::<fn(&dyn Any) -> Option<&T>>()?(self.src)
    }
}

// A dynamically-typed mutable reference ready to be cast to the type `&mut T`
// for some `T: 'static + ?Sized`.
pub struct DynCastMut<'a> { src: &'a mut dyn Any, fun: &'static dyn Any }

impl<'a> DynCastMut<'a> {
    // Constructs a `DynCastMut` that may be cast to `&mut T`, given an
    // underlying mutable reference and a function which, given the underlying
    // reference as `&mut dyn Any`, performs the casting to `&mut T`.
    //
    // Note that, for technical reasons, `fun` must be given indirectly as a
    // static reference to a function pointer to the actual function. 
    pub fn from_downcast_fn<S: Any, T: ?Sized + 'static>(
        src: &'a mut S, fun: &'static fn(&mut dyn Any) -> Option<&mut T>
    ) -> Self {
        Self{ src, fun }
    }

    // Consumes the `DynCastMut`, yielding some casting result of type `&mut T`,
    // if this is the correct type, or else `None`.
    pub fn cast<T: 'static + ?Sized>(self) -> Option<&'a mut T> {
        self.fun.downcast_ref::<fn(&mut dyn Any) -> Option<&mut T>>()?(self.src)
    }
}

// A dynamically-typed box ready to be cast to the type `Box<T>`
// for some `T: 'static + ?Sized`.
pub struct DynCastBox { src: Box<dyn Any>, fun: &'static dyn Any }

impl<'a> DynCastBox {
    // Constructs a `DynCastBox` that may be cast to `Box<T>`, given an
    // underlying box and a function which, given the underlying box as
    // `Box<dyn Any>`, performs the casting to `Box<T>`.
    //
    // Note that, for technical reasons, `fun` must be given indirectly as a
    // static reference to a function pointer to the actual function. 
    pub fn from_downcast_fn<S: Any, T: ?Sized + 'static>(
        src: Box<S>, fun: &'static fn(Box<dyn Any>) -> Option<Box<T>>
    ) -> Self {
        Self{ src, fun }
    }

    // Consumes the `DynCastBox`, yielding some casting result of type `Box<T>`,
    // if this is the correct type, or else drops it and returns `None`.
    pub fn cast<T: 'static + ?Sized>(self) -> Option<Box<T>> {
        self.fun.downcast_ref::<fn(Box<dyn Any>) -> Option<Box<T>>>()?(self.src)
    }
}

// A dynamically-typed reference-counted pointer ready to be cast to the type
// `Rc<T>` for some `T: 'static + ?Sized`.
pub struct DynCastRc { src: Rc<dyn Any>, fun: &'static dyn Any }

impl<'a> DynCastRc {
    // Constructs a `DynCastRc` that may be cast to `Rc<T>`, given an
    // underlying pointer and a function which, given the underlying pointer as
    // `Rc<dyn Any>`, performs the casting to `Rc<T>`.
    //
    // Note that, for technical reasons, `fun` must be given indirectly as a
    // static reference to a function pointer to the actual function. 
    pub fn from_downcast_fn<S: Any, T: ?Sized + 'static>(
        src: Rc<S>, fun: &'static fn(Rc<dyn Any>) -> Option<Rc<T>>
    ) -> Self {
        Self{ src, fun }
    }

    // Consumes the `DynCastRc`, yielding some casting result of type `Rc<T>`,
    // if this is the correct type, or else drops it and returns `None`.
    pub fn cast<T: 'static + ?Sized>(self) -> Option<Rc<T>> {
        self.fun.downcast_ref::<fn(Rc<dyn Any>) -> Option<Rc<T>>>()?(self.src)
    }
}

// A dynamically-typed atomically reference-counted pointer ready to be cast to
// the type `Arc<T>` for some `T: 'static + ?Sized`.
pub struct DynCastArc { src: Arc<dyn Any + Send + Sync>, fun: &'static dyn Any }

impl<'a> DynCastArc {
    // Constructs a `DynCastArc` that may be cast to `Arc<T>`, given an
    // underlying pointer and a function which, given the underlying pointer as
    // `Arc<dyn Any + Send + Sync>`, performs the casting to `Arc<T>`.
    //
    // Note that, for technical reasons, `fun` must be given indirectly as a
    // static reference to a function pointer to the actual function. 
    pub fn from_downcast_fn<S: Any + Send + Sync, T: ?Sized + 'static>(
        src: Arc<S>,
        fun: &'static fn(Arc<dyn Any + Send + Sync>) -> Option<Arc<T>>,
    ) -> Self {
        Self{ src, fun }
    }

    // Consumes the `DynCastArc`, yielding some casting result of type `Arc<T>`,
    // if this is the correct type, or else drops it and returns `None`.
    pub fn cast<T: 'static + ?Sized>(self) -> Option<Arc<T>> {
        self.fun.downcast_ref::<
            fn(Arc<dyn Any + Send + Sync>) -> Option<Arc<T>>
        >()?(self.src)
    }
}

const DYNCAST_ERR: &str
    = "The contract of `DynCast` has been broken by an implementation.";

// Extension methods for types implementing `DynCast`, providing a type-safe
// high-level interface to its functionality, intended for users to call. These
// methods may panic if the `DynCast` implementation breaks the contract.
pub trait DynCastExt: DynCast {
    // Returns `true` if `self` can be cast to type `T`, or otherwise `false`.
    fn can_cast<T: Any + ?Sized>(&self) -> bool {
        self.dyn_can_cast(TypeId::of::<T>())
    }

    // If `self` can be cast to type `T`, returns `Some` with the given shared
    // reference cast from `&Self` to `&T`, or otherwise `None`.
    fn cast_ref<T: Any + ?Sized>(&self) -> Option<&T> where {
        let res = self.dyn_cast_ref(TypeId::of::<T>())?;
        Some(res.cast::<T>().expect(DYNCAST_ERR))
    }

    // If `self` can be cast to type `T`, returns `Some` with the given mutable
    // reference cast from `&mut Self` to `&mut T`, or otherwise `None`.
    fn cast_mut<T: Any + ?Sized>(&mut self) -> Option<&mut T> {
        let res = self.dyn_cast_mut(TypeId::of::<T>())?;
        Some(res.cast::<T>().expect(DYNCAST_ERR))
    }

    // If `self` can be cast to type `T`, returns `Ok` with the given box cast
    // from `Box<Self>` to `Box<T>`, or otherwise `Err` with the original box.
    fn cast_box<T: Any + ?Sized>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if !self.can_cast::<T>() { return Err(self); }
        let res = self.dyn_cast_box(TypeId::of::<T>()).expect(DYNCAST_ERR);
        Ok(res.cast::<T>().expect(DYNCAST_ERR))
    }

    // If `self` can be cast to type `T`, returns `Ok` with the given
    // reference-counted pointer cast from `Rc<Self>` to `Rc<T>`, or otherwise
    // `Err` with the original pointer.
    fn cast_rc<T: Any + ?Sized>(self: Rc<Self>) -> Result<Rc<T>, Rc<Self>> {
        if !self.can_cast::<T>() { return Err(self); }
        let res = self.dyn_cast_rc(TypeId::of::<T>()).expect(DYNCAST_ERR);
        Ok(res.cast::<T>().expect(DYNCAST_ERR))
    }

    // If `self` can be cast to the types `T` **and** `dyn Any + Sync + Send`,
    // returns `Ok` with the given atomically reference-counted pointer cast
    // from `Arc<Self>` to `Arc<T>`, or otherwise `Err` with the original
    // pointer.
    fn cast_arc<T: Any + ?Sized>(self: Arc<Self>) -> Result<Arc<T>, Arc<Self>> {
        if !self.can_cast::<T>() { return Err(self); }
        if !self.can_cast::<dyn Any + Send + Sync>() { return Err(self); }
        let res = self.dyn_cast_arc(TypeId::of::<T>()).expect(DYNCAST_ERR);
        Ok(res.cast::<T>().expect(DYNCAST_ERR))
    }
}
impl<S> DynCastExt for S where S: DynCast + ?Sized {}
