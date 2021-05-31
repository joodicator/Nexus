use std::marker::PhantomData;
use std::any::Any;

// Immutable references to values of dynamic type.
// Used to make trait methods object-safe by eliminating type variables.
pub struct DynRef<'a> {
    // Invariant: `obj` contains a pointer of type `*const T`
    // corresponding to a valid reference of type `&'a T`,
    // for some type T: Any + ?Sized.
    obj: Box<dyn Any>,

    // Declares that this type borrows a value with lifetime `'a`.
    phantom: PhantomData<&'a ()>,
}

impl<'a> DynRef<'a> {
    pub fn new<T>(target: &'a T) -> Self
    where T: Any + ?Sized {
        Self {
            obj: Box::new(target as *const T) as Box<dyn Any>,
            phantom: PhantomData,
        }
    }

    pub fn downcast_ref<T>(&self) -> Option<&'a T>
    where T: Any + ?Sized {
        let ptr: &*const T = self.obj.downcast_ref::<*const T>()?;
        unsafe { Some(&**ptr) }
    }
}

// Mutable references of values of dynamic type.
// Used to make trait methods object-safe by eliminating type variables.
pub struct DynMut<'a> {
    // Invariant: `obj` contains a pointer of type `*mut T`
    // corresponding to a valid reference of type `&'a mut T`,
    // for some type T: Any + ?Sized.
    obj: Box<dyn Any>,

    // Declares that this type mutably borrows a value with lifetime `'a`.
    phantom: PhantomData<&'a mut ()>,
}

impl<'a> DynMut<'a> {
    pub fn new<T>(target: &'a mut T) -> Self
    where T: Any + ?Sized {
        Self {
            obj: Box::new(target as *mut T) as Box<dyn Any>,
            phantom: PhantomData,
        }
    }

    pub fn downcast_ref<T>(&self) -> Option<&'a T>
    where T: Any + ?Sized {
        let ptr: &*mut T = self.obj.downcast_ref::<*mut T>()?;
        unsafe { Some(&**ptr) }
    }

    pub fn downcast_mut<T>(&mut self) -> Option<&'a mut T>
    where T: Any + ?Sized {
        let ptr: &mut *mut T = self.obj.downcast_mut::<*mut T>()?;
        unsafe { Some(&mut **ptr) }
    }
}
