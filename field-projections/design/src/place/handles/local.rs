use std::ptr::NonNull;

use crate::{
    ops::place::{
        BorrowPlace,
        CreateHandle,
        DerefPlace,
        PlaceHandle,
        ProjectPlace,
        ReadPlace,
        ReadVariant,
        VariantPlace,
        WritePlace,
        borrowck::{
            AccessKind,
            Instant,
            Lifetime,
            Timing,
        },
    },
    place::subplace::{
        HasVariant,
        Matchable,
        Subplace,
        VariantType,
    },
};

pub struct LocalHandle<T: ?Sized> {
    ptr: *const T,
}

impl<T: ?Sized> Clone for LocalHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: ?Sized> Copy for LocalHandle<T> {}

impl<T: ?Sized> LocalHandle<T> {
    pub unsafe fn new(ptr: *const T) -> Self {
        Self { ptr }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr.cast_mut()
    }
}

impl<T: ?Sized> PlaceHandle for LocalHandle<T> {
    type Target = T;
}

unsafe impl<P, ProxyTiming> DerefPlace<ProxyTiming, Instant> for LocalHandle<P>
where
    P: CreateHandle<ProxyTiming>,
    ProxyTiming: Timing,
{
    const POINTEE_ACCESS: AccessKind = P::ACCESS;
    const POINTER_ACCESS: AccessKind = P::ACCESS;

    const SAFE: bool = true;

    unsafe fn deref_place(
        self,
    ) -> <Self::Target as CreateHandle<ProxyTiming>>::Handle {
        unsafe { P::handle_from_raw(self.as_ptr()) }
    }
}

unsafe impl<T: ?Sized + Matchable> ReadVariant for LocalHandle<T> {
    unsafe fn read_variant(self) -> &'static str {
        unsafe { T::variant_at(self.ptr) }
    }
}

unsafe impl<T, const VARIANT: &'static str> VariantPlace<VARIANT>
    for LocalHandle<T>
where
    T: ?Sized + HasVariant<VARIANT>,
{
    type ToVariant = LocalHandle<VariantType<T, VARIANT>>;

    unsafe fn cast(self) -> Self::ToVariant {
        LocalHandle { ptr: self.ptr.cast() }
    }
}

unsafe impl<S> ProjectPlace<S> for LocalHandle<S::Source>
where
    S: Subplace,
{
    type Projected = LocalHandle<S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        LocalHandle {
            ptr: unsafe { self.ptr.project_place(subplace) },
        }
    }
}

unsafe impl<T> ReadPlace for LocalHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    const SAFE: bool = true;

    unsafe fn read_place(self) -> Self::Target {
        unsafe { self.ptr.read() }
    }
}

unsafe impl<T> WritePlace for LocalHandle<T> {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    const SAFE: bool = true;

    unsafe fn write_place(self, value: Self::Target) {
        unsafe { self.ptr.cast_mut().write(value) }
    }
}

unsafe impl<'a, T> BorrowPlace<&'a T> for LocalHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> &'a T {
        unsafe { self.ptr.as_ref_unchecked() }
    }
}

unsafe impl<'a, T> BorrowPlace<&'a mut T> for LocalHandle<T> {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> &'a mut T {
        unsafe { self.ptr.cast_mut().as_mut_unchecked() }
    }
}

unsafe impl<T> BorrowPlace<NonNull<T>> for LocalHandle<T> {
    const ACCESS: AccessKind = AccessKind::Untracked;
    type Timing = Instant;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(self.ptr.cast_mut()) }
    }
}

unsafe impl<T> BorrowPlace<*const T> for LocalHandle<T> {
    const ACCESS: AccessKind = AccessKind::Untracked;
    type Timing = Instant;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> *const T {
        self.ptr
    }
}

unsafe impl<T> BorrowPlace<*mut T> for LocalHandle<T> {
    const ACCESS: AccessKind = AccessKind::Untracked;
    type Timing = Instant;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> *mut T {
        self.ptr.cast_mut()
    }
}
