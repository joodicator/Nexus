//! The [`DynCast`][trait@DynCast] trait and related items.

use std::any::{Any, TypeId};
use std::{rc::Rc, sync::Arc};
use std::marker::{Sync, Send};

mod tests;

/// Trait providing a generalised form of dynamic typing.
///
/// Extends the downcasting behaviour of [`Any`] with the ability to cast into
/// any arbitrary type supported by the implementation (not just downcasting to
/// the concrete type of an object), based on dynamic runtime type reflection.
///
/// For users, the [extension trait] [`DynCastExt`] should be imported alongside
/// `DynCast` to enable the intended high-level user interface. The methods in
/// this trait have unusual types due to the constraints of [object safety],
/// making them inconvenient for general use.
///
/// For implementors, the macro [`DynCast!`] can, and usually should, be used to
/// derive an implementation that allows casting into any trait object from a
/// finite set of declared traits.
///
/// # Examples
/// The motivating use case is *cross-casting* between different trait object
/// representations of a concrete type:
#[cfg_attr(feature = "derive", doc = "```")]
#[cfg_attr(not(feature = "derive"), doc = "```ignore")]
/// # use nxs_interface::util::dyn_cast::{DynCast, DynCastExt};
/// #
/// trait Get0: DynCast { fn get0(&self) -> &'static str; }
/// trait Get1: DynCast { fn get1(&self) -> &'static str; }
///
/// #[derive(DynCast)]
/// #[dyn_cast(base_traits(Get0, Get1))]
/// struct Concrete(&'static str, &'static str);
///
/// impl Get0 for Concrete { fn get0(&self) -> &'static str { self.0 } }
/// impl Get1 for Concrete { fn get1(&self) -> &'static str { self.1 } }
///
/// let obj = &(Concrete("zero", "one")) as &dyn Get0;
/// assert_eq!(obj.get0(), "zero");
///
/// // `obj` has no static type information indicating that its underlying
/// // concrete type also implements `Get1`. However, because `DynCast` is a
/// // supertrait of `Get0` and `Concrete`'s `DynCast` implementation declares
/// // support for casting to `Get1`, this information can be extracted at
/// // runtime:
/// let obj = obj.cast_ref::<dyn Get1>().ok_or("failed to cast to `dyn Get1`")?;
/// assert_eq!(obj.get1(), "one");
///
/// // Because `DynCast` is also a supertrait of `Get1` and `Concrete`'s
/// // `DynCast` implementation also declares supports for casting to `Get0`,
/// // we can also perform the reverse cast:
/// let obj = obj.cast_ref::<dyn Get0>().ok_or("failed to cast to `dyn Get0`")?;
/// assert_eq!(obj.get0(), "zero");
///
/// // We can also perform the simple downcasting operation already supported
/// // by Any, to recover the underlying concrete type:
/// let val = obj.cast_ref::<Concrete>().ok_or("failed to cast to `Concrete`")?;
/// assert_eq!(val.0, "zero");
/// assert_eq!(val.1, "one");
/// #
/// # Ok::<(), &str>(())
/// ```
///
/// [`Any`]: std::any::Any
/// [`DynCast!`]: macro@crate::util::dyn_cast::DynCast
/// [extension trait]: https://rust-lang.github.io/rfcs/0445-extension-trait-conventions.html
/// [object safety]: https://doc.rust-lang.org/reference/items/traits.html#object-safety
pub trait DynCast: Any {
    /// Tells whether casting to a given [`TypeId`] is is possible.
    ///
    /// Specifically, if `*self` can be cast to the type `T` for which
    /// `TypeId::of::<T>() == to`, returns `true`, or otherwise `false`.
    fn dyn_can_cast(&self, to: TypeId) -> bool;

    /// Returns a vector of the [`TypeId`]s to which casting is possible.
    ///
    /// The `TypeId`s in the returned vector are exactly those for which
    /// [`dyn_can_cast`](Self::dyn_can_cast) returns `true`.
    ///
    /// Some type IDs *may* appear multiple times in the vector, for example if
    /// an automatic derivation of `DynCast` is configured with different paths
    /// resolving to the same base trait, but this is expected to be rare.
    fn castable_types(&self) -> Vec<TypeId>;

    /// Attempts to cast a shared reference to a given [`TypeId`].
    ///
    /// If `*self` can be cast to the type `T` for which
    /// `TypeId::of::<T>() == to`, returns some [`DynCastRef`] yielding `self`
    /// as `&T`, or else `None`.
    fn dyn_cast_ref<'a>(&'a self, to: TypeId) -> Option<DynCastRef<'a>>;

    /// Attempts to cast a mutable reference to a given [`TypeId`].
    ///
    /// If `*self` can be cast to the type `T` for which
    /// `TypeId::of::<T>() == to`, returns some [`DynCastMut`] yielding `self`
    /// as `&mut T`, or else `None`.
    fn dyn_cast_mut<'a>(&'a mut self, to: TypeId) -> Option<DynCastMut<'a>>;

    /// Attempts to cast a box to a given [`TypeId`].
    ///
    /// If `*self` can be cast to the type `T` for which
    /// `TypeId::of::<T>() == to`, returns some [`DynCastBox`] yielding `self`
    /// as `Box<T>`, or else drops `self` returns `None`.
    fn dyn_cast_box(self: Box<Self>, to: TypeId) -> Option<DynCastBox>;

    /// Attempts to cast a reference-counted pointer to a given [`TypeId`].
    ///
    /// If `*self` can be cast to the type `T` for which
    /// `TypeId::of::<T>() == to`, returns some [`DynCastRc`] yielding `self`
    /// as `Rc<T>`, or else drops the reference to `*self` and returns `None`.
    fn dyn_cast_rc(self: Rc<Self>, to: TypeId) -> Option<DynCastRc>;

    /// Attempts to cast an atomically reference-counted pointer to a given
    /// [`TypeId`].
    ///
    /// If `*self` can be cast to the type `T` for which
    /// `TypeId::of::<T>() == to` **and** `*self` can be cast to
    /// `dyn Any + Sync + Send`, returns some [`DynCastArc`] yielding `self` as
    /// `Arc<T>`, or else drops the reference to `*self` and returns `None`.
    fn dyn_cast_arc(self: Arc<Self>, to: TypeId) -> Option<DynCastArc>;
}

/// The successful return type of [`DynCast::dyn_cast_ref`].
///
/// To extract the actual result, call [`DynCastRef::cast`].
pub struct DynCastRef<'a> { src: &'a dyn Any, fun: &'static dyn Any }
impl<'a> DynCastRef<'a> {
    /// Constructs a `DynCastRef` from a reference that can be upcast to
    /// `&dyn Any` and a function that can cast it thence to `&T`, so that
    /// `DynCastRef::from_any_cast_fn::<S, T>(arg, fun).cast::<T>()` is
    /// equivalent to `fun(arg as &dyn Any)`.
    pub fn from_any_cast_fn<S: Any, T: Any + ?Sized>(
        src: &'a S, fun: &'static fn(&dyn Any) -> Option<&T>,
    ) -> Self { Self { src, fun } }

    /// Returns the casted reference, given its correct referent type.
    pub fn cast<T: Any + ?Sized>(self) -> Option<&'a T> {
        self.fun.downcast_ref::<fn(&dyn Any) -> Option<&T>>()?(self.src)
    }
}

/// The successful return type of [`DynCast::dyn_cast_mut`].
///
/// To extract the actual result, call [`DynCastMut::cast`].
pub struct DynCastMut<'a> { src: &'a mut dyn Any, fun: &'static dyn Any }
impl<'a> DynCastMut<'a> {
    /// Constructs a `DynCastMut` from a reference that can be upcast to
    /// `&mut dyn Any` and a function that can cast it thence to `&mut T`,
    /// so that `DynCastMut::from_any_cast_fn::<S, T>(arg, fun).cast::<T>()` is
    /// equivalent to `fun(arg as &mut dyn Any)`.
    pub fn from_any_cast_fn<S: Any, T: Any + ?Sized>(
        src: &'a mut S, fun: &'static fn(&mut dyn Any) -> Option<&mut T>,
    ) -> Self { Self { src, fun } }

    /// Returns the casted reference, given its correct referent type.
    pub fn cast<T: Any + ?Sized>(self) -> Option<&'a mut T> {
        self.fun.downcast_ref::<fn(&mut dyn Any) -> Option<&mut T>>()?(self.src)
    }
}

/// The successful return type of [`DynCast::dyn_cast_box`].
///
/// To extract the actual result, call [`DynCastBox::cast`].
pub struct DynCastBox { src: Box<dyn Any>, fun: &'static dyn Any }
impl DynCastBox {
    /// Constructs a `DynCastBox` from a box that can be upcast to
    /// `Box<dyn Any>` and a function that can cast it thence to `Box<T>`,
    /// so that `DynCastMut::from_any_cast_fn::<S, T>(arg, fun).cast::<T>()` is
    /// equivalent to `fun(arg as Box<dyn Any>)`.
    pub fn from_any_cast_fn<S: Any, T: Any + ?Sized>(
        src: Box<S>, fun: &'static fn(Box<dyn Any>) -> Option<Box<T>>,
    ) -> Self { Self { src, fun } }

    /// Returns the casted box, given its correct referent type.
    pub fn cast<T: Any + ?Sized>(self) -> Option<Box<T>> {
        self.fun.downcast_ref::<fn(Box<dyn Any>) -> Option<Box<T>>>()?(self.src)
    }
}

/// The successful return type of [`DynCast::dyn_cast_rc`].
///
/// To extract the actual result, call [`DynCastRc::cast`].
pub struct DynCastRc { src: Rc<dyn Any>, fun: &'static dyn Any }
impl DynCastRc {
    /// Constructs a `DynCastRc` from a pointer that can be upcast to
    /// `Rc<dyn Any>` and a function that can cast it thence to `Rc<T>`, so
    /// that `DynCastMut::from_any_cast_fn::<S, T>(arg, fun).cast::<T>()` is
    /// equivalent to `fun(arg as Rc<dyn Any>)`.
    pub fn from_any_cast_fn<S: Any, T: Any + ?Sized>(
        src: Rc<S>, fun: &'static fn(Rc<dyn Any>) -> Option<Rc<T>>,
    ) -> Self { Self { src, fun } }

    /// Returns the casted pointer, given its correct referent type.
    pub fn cast<T: Any + ?Sized>(self) -> Option<Rc<T>> {
        self.fun.downcast_ref::<fn(Rc<dyn Any>) -> Option<Rc<T>>>()?(self.src)
    }
}

/// The successful return type of [`DynCast::dyn_cast_arc`].
///
/// To extract the actual result, call [`DynCastArc::cast`].
pub struct DynCastArc { src: Arc<dyn Any + Send + Sync>, fun: &'static dyn Any }
impl DynCastArc {
    /// Constructs a `DynCastArc` from a pointer that can be upcast to
    /// `Arc<dyn Any + Send + Sync>` and a function that can cast it thence
    /// to `Arc<T>`, so that
    /// `DynCastMut::from_any_cast_fn::<S, T>(arg, fun).cast::<T>()` is
    /// equivalent to `fun(arg as Arc<dyn Any>)`.
    pub fn from_any_cast_fn<S: Any + Send + Sync, T: Any + ?Sized>(
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

/// User-friendly extension methods for [`DynCast`][trait@DynCast].
/// 
/// This extension trait contains non-object-safe generic methods necessary for
/// users to interact with the `DynCast` trait in a type-safe manner. A blanket
/// implementation makes it available on all `dyn DynCast` trait objects.
///
/// # Examples
/// See [`DynCast#examples`][trait@DynCast#examples].
pub trait DynCastExt: DynCast {
    /// Tells whether casting to a given type argument is possible.
    ///
    /// Specifically, returns `true` if `*self` can be cast to type `T`, or
    /// otherwise `false`. Generalises [`<dyn Any>::is`](https://doc.rust-lang.org/nightly/std/any/trait.Any.html#method.is).
    fn can_cast<T: Any + ?Sized>(&self) -> bool {
        self.dyn_can_cast(TypeId::of::<T>())
    }

    /// Attempts to cast a shared reference to a given type.
    /// 
    /// If `*self` can be cast to type `T`, returns `Some` with the given shared
    /// reference cast from `&Self` to `&T`, or otherwise `None`. Generalises
    /// [`<dyn Any>::downcast_ref`](https://doc.rust-lang.org/nightly/std/any/trait.Any.html#method.downcast_ref).
    fn cast_ref<T: Any + ?Sized>(&self) -> Option<&T> where {
        let res = self.dyn_cast_ref(TypeId::of::<T>())?;
        Some(res.cast::<T>().expect(DYNCAST_ERR))
    }

    /// Attempts to cast a mutable reference to a given type.
    ///
    /// If `*self` can be cast to type `T`, returns `Some` with the given mutable
    /// reference cast from `&mut Self` to `&mut T`, or otherwise `None`.
    /// Generalises [`<dyn Any>::downcast_mut`](https://doc.rust-lang.org/nightly/std/any/trait.Any.html#method.downcast_mut).
    fn cast_mut<T: Any + ?Sized>(&mut self) -> Option<&mut T> {
        let res = self.dyn_cast_mut(TypeId::of::<T>())?;
        Some(res.cast::<T>().expect(DYNCAST_ERR))
    }

    /// Attempts to cast a box to a given type.
    ///
    /// If `*self` can be cast to type `T`, returns `Ok` with the given box cast
    /// from `Box<Self>` to `Box<T>`, or otherwise `Err` with the original box.
    /// Generalises [`<Box<dyn Any>>::downcast`](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.downcast).
    fn cast_box<T: Any + ?Sized>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if !self.can_cast::<T>() { return Err(self); }
        let res = self.dyn_cast_box(TypeId::of::<T>()).expect(DYNCAST_ERR);
        Ok(res.cast::<T>().expect(DYNCAST_ERR))
    }

    /// Attempts to cast a reference-counted pointer to a given type.
    ///
    /// If `*self` can be cast to type `T`, returns `Ok` with the given
    /// reference-counted pointer cast from `Rc<Self>` to `Rc<T>`, or otherwise
    /// `Err` with the original pointer. Generalises
    /// [`<Rc<dyn Any>>::downcast`](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.downcast).
    fn cast_rc<T: Any + ?Sized>(self: Rc<Self>) -> Result<Rc<T>, Rc<Self>> {
        if !self.can_cast::<T>() { return Err(self); }
        let res = self.dyn_cast_rc(TypeId::of::<T>()).expect(DYNCAST_ERR);
        Ok(res.cast::<T>().expect(DYNCAST_ERR))
    }

    /// Attempts to cast an atomically reference-counted pointer to a given
    /// type.
    ///
    /// If `*self` can be cast to the types `T` **and** `dyn Any + Sync + Send`,
    /// returns `Ok` with the given atomically reference-counted pointer cast
    /// from `Arc<Self>` to `Arc<T>`, or otherwise `Err` with the original
    /// pointer. Generalises [`<Arc<dyn Any + Send + Sync>>::downcast`](https://doc.rust-lang.org/std/sync/struct.Arc.html#method.downcast).
    fn cast_arc<T: Any + ?Sized>(self: Arc<Self>) -> Result<Arc<T>, Arc<Self>> {
        if !self.can_cast::<T>() { return Err(self); }
        if !self.can_cast::<dyn Any + Send + Sync>() { return Err(self); }
        let res = self.dyn_cast_arc(TypeId::of::<T>()).expect(DYNCAST_ERR);
        Ok(res.cast::<T>().expect(DYNCAST_ERR))
    }
}
impl<S> DynCastExt for S where S: DynCast + ?Sized {}

#[cfg(feature = "derive")]
/// Derives an instance of the [`DynCast`] trait.
///
/// # Usage
/// ```text
/// DynCast!(ImplType[, base_traits(B1,B2,...,Bm)][, auto_traits(A1,A2,...,An)]);
/// ```
/// where:
/// * Square brackets indicate optional parts of the syntax, and should not be
///   included in the macro invocation.
/// 
/// * `ImplType` is a type, usually the name of a `struct` or `enum`.
///
/// * Each `Bi` is a trait implemented by `ImplType` usable as the *base trait*
///   of a [trait object], **except** for [`DynCast`] or [`Any`]. If the
///   `base_traits` key is not specified, it defaults to `base_traits()`.
///   
/// * Each `Aj` is an [auto trait] implemented by `ImplType`. Auto traits must
///   be specified by one of the identifiers `Send`, `Sync`, `Unpin`,
///   `UnwindSafe`, or `RefUnwindsafe`: paths or type aliases are not accepted,
///   and these identifiers refer to the standard traits they name regardless
///   of what is in scope at the call site. If the `auto_traits` key is not
///   specified, it defaults to `auto_traits(Send, Sync)`.
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
/// Minimal usage:
/// ```
/// use std::any::Any;
/// use nxs_interface::util::dyn_cast::{DynCast, DynCastExt};
///
/// #[derive(DynCast)]
/// struct Struct0 { /* ... */ }
///
/// let obj = &(Struct0 { /* ... */ }) as &dyn DynCast;
///
/// // The following casts are always derived (ignoring auto traits):
/// assert!(obj.can_cast::<Struct0>());
/// assert!(obj.can_cast::<dyn Any>());
/// assert!(obj.can_cast::<dyn DynCast>());
/// ```
///
/// Possible combinations of base and auto traits:
/// ```
/// # use std::any::Any;
/// # use nxs_interface::util::dyn_cast::{DynCast, DynCastExt};
/// use std::{
///     marker::Unpin, any::TypeId, collections::HashSet, iter::FromIterator,
/// };
///
/// trait Trait1 { /* ... */ }
/// trait Trait2 { /* ... */ }
/// 
/// #[derive(DynCast)]
/// #[dyn_cast(base_traits(Trait1), auto_traits(Send, Unpin))]
/// struct Struct1 { /* ... */ }
///
/// impl Trait1 for Struct1 { /* ... */ }
///
/// let obj = &(Struct1 { /* ... */ }) as &dyn DynCast;
///
/// // `obj` can be cast to exactly the following 13 types:
/// let castable: HashSet<TypeId> = obj.castable_types().into_iter().collect();
/// assert_eq!(castable, HashSet::<TypeId>::from_iter([
///     TypeId::of::<Struct1>(),
///     TypeId::of::<dyn Trait1>(),
///     TypeId::of::<dyn Trait1 + Send>(),
///     TypeId::of::<dyn Trait1 + Unpin>(),
///     TypeId::of::<dyn Trait1 + Send + Unpin>(),
///     TypeId::of::<dyn DynCast>(),
///     TypeId::of::<dyn DynCast + Send>(),
///     TypeId::of::<dyn DynCast + Unpin>(),
///     TypeId::of::<dyn DynCast + Send + Unpin>(),
///     TypeId::of::<dyn Any>(),
///     TypeId::of::<dyn Any + Send>(),
///     TypeId::of::<dyn Any + Unpin>(),
///     TypeId::of::<dyn Any + Send + Unpin>(),
/// ]));
///
/// // And not, for example, this type, which was absent from the declaration:
/// assert!(!obj.can_cast::<dyn Trait2>());
/// ```
///
/// Declaring many base traits:
/// ```
/// # use nxs_interface::util::dyn_cast::{DynCast, DynCastExt};
///
/// # trait Trait1 { /* ... */ }
/// # trait Trait2 { /* ... */ }
/// trait Trait3 { /* ... */ }
/// trait Trait4 { /* ... */ }
/// #
/// #[derive(DynCast)]
/// #[dyn_cast(base_traits(Trait1, Trait2, Trait3, Trait4))]
/// struct Struct2 { /* ... */ }
/// impl Trait1 for Struct2 { /* ... */ }
/// impl Trait2 for Struct2 { /* ... */ }
/// impl Trait3 for Struct2 { /* ... */ }
/// impl Trait4 for Struct2 { /* ... */ }
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
/// [`DynCast`]: trait@crate::util::dyn_cast::DynCast
pub use nxs_interface_macros::DynCast;
