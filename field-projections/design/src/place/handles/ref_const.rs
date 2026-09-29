use std::{
    marker::PhantomData,
    ptr::NonNull,
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
        },
    },
    place::subplace::Subplace,
};

pub struct RefHandle<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _lt: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> PlaceProxy for &'a T {
    type Target = T;
}

unsafe impl<'a, T: ?Sized> CreateHandle<Instant> for &'a T {
    type Handle = RefHandle<'a, T>;

    const ACCESS: AccessKind = AccessKind::Shared;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        let ptr: *const *const T = this.cast::<*const T>();
        let handle = unsafe { *ptr };
        let handle = handle.cast_mut();
        RefHandle {
            ptr: unsafe { NonNull::new_unchecked(handle) },
            _lt: PhantomData,
        }
    }
}

impl<'a, T: ?Sized> PlaceHandle for RefHandle<'a, T> {
    type Target = T;
}

unsafe impl<'a, S: Subplace<Target: 'a>> ProjectPlace<S>
    for RefHandle<'a, S::Source>
{
    type Projected = RefHandle<'a, S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        RefHandle {
            ptr: unsafe { self.ptr.project_place(subplace) },
            _lt: PhantomData,
        }
    }
}

unsafe impl<'a, ProxyTiming, P: ?Sized> DerefPlace<ProxyTiming, Instant>
    for RefHandle<'a, P>
where
    P: CreateHandle<ProxyTiming>,
    ProxyTiming: Timing,
{
    const POINTEE_ACCESS: AccessKind = P::ACCESS;
    const POINTER_ACCESS: AccessKind = AccessKind::Shared;
    const SAFE: bool = true;

    unsafe fn deref_place(
        self,
    ) -> <Self::Target as CreateHandle<ProxyTiming>>::Handle {
        unsafe { P::handle_from_raw(self.ptr.as_ptr()) }
    }
}

unsafe impl<'a, 'b, T> BorrowPlace<&'b T> for RefHandle<'a, T>
where
    'a: 'b,
{
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Lifetime<'b>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> &'b T {
        unsafe { &*self.ptr.as_ptr() }
    }
}

unsafe impl<'a, T> ReadPlace for RefHandle<'a, T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    const SAFE: bool = true;

    unsafe fn read_place(self) -> Self::Target {
        unsafe { self.ptr.read() }
    }
}
