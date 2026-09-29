use std::{
    alloc::Layout,
    ptr::NonNull,
    sync::{
        Arc,
        atomic::{
            AtomicUsize,
            Ordering,
        },
    },
};

use crate::{
    ops::place::{
        BorrowPlace,
        CreateHandle,
        PlaceHandle,
        PlaceProxy,
        borrowck::{
            AccessKind,
            Instant,
            Lifetime,
        },
    },
    sync::arc_ref::ArcRef,
};

#[repr(C, align(2))]
struct ArcInner<T: ?Sized> {
    strong: AtomicUsize,
    weak: AtomicUsize,
    data: T,
}

#[repr(C, align(2))]
pub(super) struct ArcHead {
    pub(super) strong: AtomicUsize,
    pub(super) weak: AtomicUsize,
}

pub struct ArcHandle<T: ?Sized>(NonNull<ArcInner<T>>);

impl<T: ?Sized> PlaceProxy for Arc<T> {
    type Target = T;
}

unsafe impl<T: ?Sized> CreateHandle<Instant> for Arc<T> {
    type Handle = ArcHandle<T>;

    const ACCESS: AccessKind = AccessKind::Shared;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        let ptr: *const NonNull<ArcInner<T>> = this.cast();
        let ptr: NonNull<ArcInner<T>> = unsafe { ptr.read() };
        ArcHandle(ptr)
    }
}

impl<T: ?Sized> PlaceHandle for ArcHandle<T> {
    type Target = T;
}

unsafe impl<'a, T: ?Sized> BorrowPlace<&'a T> for ArcHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> &'a T {
        let ptr = self.0.as_ptr();
        unsafe { &(*ptr).data }
    }
}

unsafe impl<T: ?Sized> BorrowPlace<*const T> for ArcHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Instant;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> *const T {
        let ptr = self.0.as_ptr();
        unsafe { &raw const (*ptr).data }
    }
}

unsafe impl<T: ?Sized> BorrowPlace<NonNull<T>> for ArcHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Instant;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> NonNull<T> {
        let ptr = self.0.as_ptr();
        unsafe { NonNull::new_unchecked(&raw mut (*ptr).data) }
    }
}

unsafe impl<T: ?Sized> BorrowPlace<ArcRef<T>> for ArcHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Instant;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> ArcRef<T> {
        let data: *mut ArcInner<T> = self.0.as_ptr();
        let data: *mut T = unsafe { &raw mut (*data).data };
        let layout = unsafe { Layout::for_value_raw(data) };
        let data = unsafe { NonNull::new_unchecked(data) };
        let head: NonNull<ArcHead> = self.0.cast();
        unsafe {
            let strong = &(*head.as_ptr()).strong;
            let old = strong.fetch_add(1, Ordering::Relaxed);
            assert_ne!(old, 0);
        }
        ArcRef { head, layout, data }
    }
}
