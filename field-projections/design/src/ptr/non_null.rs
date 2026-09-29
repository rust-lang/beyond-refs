use std::ptr::NonNull;

use crate::{
    ops::place::{
        BorrowPlace,
        CreateHandle,
        PlaceHandle,
        PlaceProxy,
        ProjectPlace,
        ReadPlace,
        WritePlace,
        borrowck::{
            AccessKind,
            Instant,
            Lifetime,
        },
    },
    place::Subplace,
};

impl<T: ?Sized> PlaceProxy for NonNull<T> {
    type Target = T;
}

unsafe impl<T: ?Sized> CreateHandle<Instant> for NonNull<T> {
    type Handle = Self;

    const ACCESS: AccessKind = AccessKind::Untracked;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        unsafe { *this }
    }
}

impl<T: ?Sized> PlaceHandle for NonNull<T> {
    type Target = T;
}

unsafe impl<S: Subplace> ProjectPlace<S> for NonNull<S::Source> {
    type Projected = NonNull<S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let ptr: *mut S::Source = self.as_ptr();
        let ptr: *mut S::Target = unsafe { ptr.project_place(subplace) };
        unsafe { NonNull::new_unchecked(ptr) }
    }
}

unsafe impl<'a, T: ?Sized> BorrowPlace<&'a T> for NonNull<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Lifetime<'a>;
    const SAFE: bool = false;

    unsafe fn borrow(self) -> &'a T {
        unsafe { self.as_ref() }
    }
}

unsafe impl<'a, T: ?Sized> BorrowPlace<&'a mut T> for NonNull<T> {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    type Timing = Lifetime<'a>;
    const SAFE: bool = false;

    unsafe fn borrow(mut self) -> &'a mut T {
        unsafe { self.as_mut() }
    }
}

unsafe impl<T: ?Sized> BorrowPlace<*const T> for NonNull<T> {
    const ACCESS: AccessKind = AccessKind::Untracked;
    type Timing = Instant;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> *const T {
        self.as_ptr()
    }
}

unsafe impl<T: ?Sized> BorrowPlace<*mut T> for NonNull<T> {
    const ACCESS: AccessKind = AccessKind::Untracked;
    type Timing = Instant;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> *mut T {
        self.as_ptr()
    }
}

unsafe impl<T: ?Sized> BorrowPlace<NonNull<T>> for NonNull<T> {
    const ACCESS: AccessKind = AccessKind::Untracked;
    type Timing = Instant;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> NonNull<T> {
        self
    }
}

unsafe impl<T> WritePlace for NonNull<T> {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    const SAFE: bool = false;

    unsafe fn write_place(self, value: Self::Target) {
        unsafe { self.write(value) }
    }
}

unsafe impl<T> ReadPlace for NonNull<T> {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    const SAFE: bool = false;

    unsafe fn read_place(self) -> Self::Target {
        unsafe { self.read() }
    }
}
