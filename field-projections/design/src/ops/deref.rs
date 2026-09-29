use std::{
    marker::PhantomData,
    ops::{
        Deref,
        DerefMut,
    },
    ptr::NonNull,
};

use crate::{
    Metadata,
    ops::place::{
        PlaceHandle,
        ProjectPlace,
        ReadMetadata,
        ReadPlace,
        WritePlace,
        borrowck::AccessKind,
        subplace::Subplace,
    },
};

/*
impl<P: Deref> ProxyPlace for P {
    type Handle = DerefHandle<P, P::Target>;
}
*/

pub struct DerefHandle<P: Deref, T: ?Sized> {
    ptr: NonNull<P>,
    offset: usize,
    _target: PhantomData<T>,
}

impl<P: Deref, T: ?Sized> Clone for DerefHandle<P, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<P: Deref, T: ?Sized> Copy for DerefHandle<P, T> {}

impl<P: Deref, T> DerefHandle<P, T> {
    fn get_ref<'a>(self) -> &'a T {
        unsafe {
            let ptr = self.ptr.as_ref();
            let ptr: *const P::Target = ptr.deref();
            let ptr: *const () = ptr.cast();
            let ptr: *const () = ptr.byte_add(self.offset);
            let ptr: *const T = ptr.cast();
            ptr.as_ref_unchecked()
        }
    }

    fn get_mut<'a>(mut self) -> &'a mut T
    where
        P: DerefMut,
    {
        unsafe {
            let ptr = self.ptr.as_mut();
            let ptr: *mut P::Target = ptr.deref_mut();
            let ptr: *mut () = ptr.cast();
            let ptr: *mut () = ptr.byte_add(self.offset);
            let ptr: *mut T = ptr.cast();
            ptr.as_mut_unchecked()
        }
    }
}

impl<P: Deref, T: ?Sized> PlaceHandle for DerefHandle<P, T> {
    type Target = T;
}

impl<P, T> ReadPlace for DerefHandle<P, T>
where
    P: Deref,
    T: Copy,
{
    const ACCESS: AccessKind = AccessKind::Shared;
    const SAFE: bool = true;

    unsafe fn read_place(self) -> Self::Target {
        *self.get_ref()
    }
}

impl<S, P> ProjectPlace<S> for DerefHandle<P, S::Source>
where
    S: Subplace,
    P: Deref,
{
    type Projected = DerefHandle<P, S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let meta = ReadMetadata::metadata(self);
        DerefHandle {
            ptr: self.ptr,
            offset: self.offset + subplace.offset(meta).0,
            _target: PhantomData,
        }
    }
}

impl<P, T> ReadMetadata for DerefHandle<P, T>
where
    P: Deref,
    T: ?Sized,
{
    fn metadata(self) -> Metadata<Self::Target> {
        todo!()
    }
}

impl<P, T> WritePlace for DerefHandle<P, T>
where
    P: DerefMut,
{
    const ACCESS: AccessKind = AccessKind::Exclusive;
    const SAFE: bool = true;

    unsafe fn write_place(self, value: Self::Target) {
        *self.get_mut() = value;
    }
}

/*
/// `*const Self` -> `Self::Handle`
///
/// LocalHandle<Self> -> Self::Handle
///
/// ```
/// fn bar(&mut self) -> &U;
///
/// fn foo(self) {
///     let val = self.bar();
///     let y = *self; // borrowck error
///     use_val(val);
///     // desugared to
///     let y = unsafe {
///         let hdl = LocalHandle::new(&raw const self);
///         let hdl = DerefPlace::deref_place(hdl);
///     };
/// }
/// ```
impl<P: Deref> ops::DerefHandle for P {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Instant;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        DerefHandle {
            ptr: NonNull::new(this.cast_mut()).unwrap(),
            offset: 0,
            _target: PhantomData,
        }
    }
}

*/
