use std::ptr;

use crate::{
    ops::place::{
        CreateHandle,
        DerefPlace,
        DropHusk,
        DropPlace,
        MovePlace,
        PlaceHandle,
        PlaceProxy,
        ProjectPlace,
        ReadMetadata,
        ReadPlace,
        WritePlace,
        borrowck::{
            AccessKind,
            Instant,
            Timing,
        },
    },
    place::Subplace,
    ptr::Metadata,
};

impl<T: ?Sized> PlaceProxy for *mut T {
    type Target = T;
}

unsafe impl<T: ?Sized> CreateHandle<Instant> for *mut T {
    type Handle = Self;

    const ACCESS: AccessKind = AccessKind::Untracked;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        unsafe { *this }
    }
}

impl<T: ?Sized> PlaceHandle for *mut T {
    type Target = T;
}

unsafe impl<T> ReadPlace for *mut T {
    const ACCESS: AccessKind = AccessKind::Untracked;
    const SAFE: bool = false;

    unsafe fn read_place(self) -> Self::Target {
        unsafe { self.read() }
    }
}

unsafe impl<T: ?Sized> ReadMetadata for *mut T {
    fn metadata(self) -> Metadata<Self::Target> {
        ptr::metadata(self)
    }
}

unsafe impl<T> MovePlace for *mut T {}

unsafe impl<T> WritePlace for *mut T {
    const ACCESS: AccessKind = AccessKind::Untracked;
    const SAFE: bool = false;

    unsafe fn write_place(self, value: Self::Target) {
        unsafe { self.write(value) }
    }
}

unsafe impl<T> DropPlace for *mut T {
    unsafe fn drop_place(self) {
        unsafe { self.drop_in_place() }
    }
}

unsafe impl<T> DropHusk for *mut T {
    unsafe fn drop_husk(_: *mut Self) {}
}

unsafe impl<S: Subplace> ProjectPlace<S> for *mut S::Source {
    type Projected = *mut S::Target;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let meta: Metadata<S::Source> = self.metadata();
        let thin: *mut () = self.cast();
        let (offset, meta) = subplace.offset(meta);
        let thin = unsafe { thin.byte_add(offset) };
        ptr::from_raw_parts_mut(thin, meta)
    }
}

unsafe impl<P, ProxyTiming> DerefPlace<ProxyTiming, Instant> for *mut P
where
    P: ?Sized + CreateHandle<ProxyTiming>,
    ProxyTiming: Timing,
{
    const POINTEE_ACCESS: AccessKind = P::ACCESS;
    const POINTER_ACCESS: AccessKind = AccessKind::Untracked;

    const SAFE: bool = false;

    unsafe fn deref_place(
        self,
    ) -> <Self::Target as CreateHandle<ProxyTiming>>::Handle {
        unsafe { P::handle_from_raw(self) }
    }
}
