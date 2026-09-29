use std::ptr::NonNull;

use crate::ops::place::{
    BorrowPlace,
    CreateHandle,
    PlaceHandle,
    PlaceProxy,
    borrowck::{
        AccessKind,
        Instant,
        Lifetime,
    },
};

impl<T> PlaceProxy for Vec<T> {
    type Target = [T];
}

unsafe impl<T> CreateHandle<Instant> for Vec<T> {
    type Handle = VecHandle<T>;

    const ACCESS: AccessKind = AccessKind::Shared;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        let this = this.cast_mut();
        // FIXME(unsound)
        let vec = unsafe { &mut *this };
        let len = vec.len();
        let ptr = Vec::as_non_null(vec);
        let ptr = NonNull::slice_from_raw_parts(ptr, len);
        VecHandle { ptr }
    }
}

pub struct VecHandle<T> {
    ptr: NonNull<[T]>,
}

impl<T> PlaceHandle for VecHandle<T> {
    type Target = [T];
}

unsafe impl<'a, T> BorrowPlace<&'a [T]> for VecHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> &'a [T] {
        unsafe { self.ptr.as_ref() }
    }
}
