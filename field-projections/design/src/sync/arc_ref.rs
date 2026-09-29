use std::{
    alloc::{
        Layout,
        dealloc,
    },
    ptr::NonNull,
    sync::atomic::Ordering,
};

use crate::{
    ops::place::{
        BorrowPlace,
        CreateHandle,
        DerefPlace,
        PlaceHandle,
        PlaceProxy,
        ProjectPlace,
        ReadPlace,
        borrowck::{
            AccessKind,
            Instant,
            Lifetime,
            Timing,
            UntilDrop,
        },
    },
    place::Subplace,
    sync::arc::ArcHead,
};

/// Reference into an [`Arc<T>`].
///
/// [`Arc<T>`]: std::sync::Arc
pub struct ArcRef<T: ?Sized> {
    pub(super) head: NonNull<ArcHead>,
    pub(super) layout: Layout,
    pub(super) data: NonNull<T>,
}

impl<T: ?Sized> Drop for ArcRef<T> {
    fn drop(&mut self) {
        let strong = unsafe { &(*self.head.as_ptr()).strong };
        let old = strong.fetch_sub(1, Ordering::Relaxed);
        if old == 1 {
            let layout = self.layout;
            let ptr: *mut u8 = self.head.as_ptr().cast();
            unsafe { dealloc(ptr, layout) };
        }
    }
}

impl<T: ?Sized> PlaceProxy for ArcRef<T> {
    type Target = T;
}

unsafe impl<T: ?Sized> CreateHandle<Instant> for ArcRef<T> {
    type Handle = ArcRefHandle<T>;

    const ACCESS: AccessKind = AccessKind::Shared;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        let head = unsafe { (*this).head };
        let layout = unsafe { (*this).layout };
        let data = unsafe { (*this).data };
        ArcRefHandle { head, layout, data }
    }
}

pub struct ArcRefHandle<T: ?Sized> {
    pub(super) head: NonNull<ArcHead>,
    pub(super) layout: Layout,
    pub(super) data: NonNull<T>,
}

impl<T: ?Sized> PlaceHandle for ArcRefHandle<T> {
    type Target = T;
}

unsafe impl<T> ReadPlace for ArcRefHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    const SAFE: bool = true;

    unsafe fn read_place(self) -> Self::Target {
        unsafe { self.data.read() }
    }
}

unsafe impl<S: Subplace> ProjectPlace<S> for ArcRefHandle<S::Source> {
    type Projected = ArcRefHandle<S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let Self { head, layout, data } = self;
        ArcRefHandle {
            head,
            layout,
            data: unsafe { data.project_place(subplace) },
        }
    }
}

unsafe impl<T: ?Sized> BorrowPlace<ArcRef<T>> for ArcRefHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Instant;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> ArcRef<T> {
        let ArcRefHandle { head, layout, data } = self;
        unsafe {
            let strong = &(*head.as_ptr()).strong;
            let old = strong.fetch_add(1, Ordering::Relaxed);
            assert_ne!(old, 0);
        }
        ArcRef { head, layout, data }
    }
}

unsafe impl<T: ?Sized> BorrowPlace<*const T> for ArcRefHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Instant;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> *const T {
        self.data.as_ptr()
    }
}

unsafe impl<T: ?Sized> BorrowPlace<NonNull<T>> for ArcRefHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Instant;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> NonNull<T> {
        self.data
    }
}

unsafe impl<'a, T: ?Sized> BorrowPlace<&'a T> for ArcRefHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> &'a T {
        unsafe { self.data.as_ref() }
    }
}

unsafe impl<T, ProxyTiming> DerefPlace<ProxyTiming, UntilDrop>
    for ArcRefHandle<T>
where
    T: ?Sized + CreateHandle<ProxyTiming>,
    ProxyTiming: Timing,
{
    const POINTEE_ACCESS: AccessKind = T::ACCESS;
    const POINTER_ACCESS: AccessKind = AccessKind::Shared;
    const SAFE: bool = true;

    unsafe fn deref_place(
        self,
    ) -> <Self::Target as CreateHandle<ProxyTiming>>::Handle {
        let ptr = self.data.as_ptr();
        unsafe { T::handle_from_raw(ptr) }
    }
}
