use std::{
    alloc::{
        Layout,
        dealloc,
    },
    marker::PhantomCovariant,
    ptr::NonNull,
};

use crate::{
    ops::place::{
        BorrowPlace,
        CreateHandle,
        DropHusk,
        DropPlace,
        MovePlace,
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

pub struct BoxHandle<T: ?Sized> {
    ptr: NonNull<T>,
    _variance: PhantomCovariant<T>,
}

impl<T: ?Sized> PlaceProxy for Box<T> {
    type Target = T;
}

unsafe impl<T: ?Sized> CreateHandle<Instant> for Box<T> {
    type Handle = BoxHandle<T>;
    const ACCESS: AccessKind = AccessKind::Shared;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        let this: *const NonNull<T> = this.cast();
        BoxHandle {
            ptr: unsafe { *this },
            _variance: PhantomCovariant::new(),
        }
    }
}

impl<T: ?Sized> PlaceHandle for BoxHandle<T> {
    type Target = T;
}

unsafe impl<S: Subplace> ProjectPlace<S> for BoxHandle<S::Source> {
    type Projected = BoxHandle<S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        BoxHandle {
            ptr: unsafe { self.ptr.project_place(subplace) },
            _variance: PhantomCovariant::new(),
        }
    }
}

unsafe impl<T> WritePlace for BoxHandle<T> {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    const SAFE: bool = true;

    unsafe fn write_place(self, value: Self::Target) {
        unsafe { self.ptr.write(value) }
    }
}

unsafe impl<T> ReadPlace for BoxHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    const SAFE: bool = true;

    unsafe fn read_place(self) -> Self::Target {
        unsafe { self.ptr.read() }
    }
}

unsafe impl<T> MovePlace for BoxHandle<T> {}

unsafe impl<T> DropPlace for BoxHandle<T> {
    unsafe fn drop_place(self) {
        unsafe { self.ptr.drop_in_place() };
    }
}

unsafe impl<T: ?Sized> DropHusk for Box<T> {
    unsafe fn drop_husk(this: *mut Self) {
        let ptr: *mut NonNull<T> = this.cast();
        let ptr = unsafe { *ptr };
        let layout = unsafe { Layout::for_value_raw(ptr.as_ptr()) };
        unsafe { dealloc(ptr.as_ptr().cast(), layout) };
    }
}

unsafe impl<'a, T: ?Sized> BorrowPlace<&'a mut T> for BoxHandle<T> {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(mut self) -> &'a mut T {
        unsafe { self.ptr.as_mut() }
    }
}
