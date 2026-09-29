use std::ptr;

use crate::{
    ops::place::{
        CreateHandle,
        DerefPlace,
        MovePlace,
        PlaceHandle,
        PlaceProxy,
        ProjectPlace,
        ReadMetadata,
        ReadPlace,
        borrowck::{
            AccessKind,
            Instant,
            Timing,
        },
    },
    place::Subplace,
    ptr::Metadata,
};

impl<T: ?Sized> PlaceProxy for *const T {
    type Target = T;
}

unsafe impl<T: ?Sized> CreateHandle<Instant> for *const T {
    type Handle = Self;

    const ACCESS: AccessKind = AccessKind::Untracked;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        unsafe { *this }
    }
}

impl<T: ?Sized> PlaceHandle for *const T {
    type Target = T;
}

unsafe impl<T> ReadPlace for *const T {
    const ACCESS: AccessKind = AccessKind::Untracked;
    const SAFE: bool = false;

    unsafe fn read_place(self) -> Self::Target {
        unsafe { self.read() }
    }
}

unsafe impl<T: ?Sized> ReadMetadata for *const T {
    fn metadata(self) -> Metadata<Self::Target> {
        ptr::metadata(self)
    }
}

unsafe impl<T> MovePlace for *const T {}

unsafe impl<S: Subplace> ProjectPlace<S> for *const S::Source {
    type Projected = *const S::Target;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let meta: Metadata<S::Source> = self.metadata();
        let thin: *const () = self.cast();
        let (offset, meta) = subplace.offset(meta);
        let thin = unsafe { thin.byte_add(offset) };
        ptr::from_raw_parts(thin, meta)
    }
}

unsafe impl<P, ProxyTiming> DerefPlace<ProxyTiming, Instant> for *const P
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
