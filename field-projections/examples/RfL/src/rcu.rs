use std::{
    pin::UnsafePinned,
    ptr,
    sync::atomic::{
        AtomicPtr,
        Ordering,
    },
};

use crate::{
    bindings,
    mutex::InsideOfMutex,
    overwrite::{
        Overwrite,
        Shield,
    },
};

pub struct Rcu<P: RcuPointer> {
    ptr: UnsafePinned<AtomicPtr<P::Target>>,
}

impl<P: RcuPointer> !Overwrite for Rcu<P> {}

pub struct RcuGuard(());

pub fn read_lock() -> RcuGuard {
    RcuGuard(())
}

pub unsafe trait RcuPointer: Sized {
    type Target;

    fn into_raw(this: Self) -> *mut Self::Target;

    unsafe fn from_raw(raw: *mut Self::Target) -> Self;
}

impl<P: RcuPointer> Drop for Rcu<P> {
    fn drop(&mut self) {
        let ptr = self.ptr.get();
        let ptr = unsafe { ptr.as_ref_unchecked() };
        let old = ptr.load(Ordering::Relaxed);
        ptr.store(ptr::null_mut(), Ordering::Relaxed);
        drop(unsafe { P::from_raw(old) });
    }
}

impl<P: RcuPointer> Rcu<P> {
    pub fn new(pointer: P) -> Self {
        Self {
            ptr: UnsafePinned::new(AtomicPtr::new(P::into_raw(pointer))),
        }
    }

    pub unsafe fn read_raw<'a>(
        this: *const Self,
        guard: &'a RcuGuard,
    ) -> &'a P::Target {
        let _ = guard;
        let ptr = UnsafePinned::raw_get(unsafe { &raw const (*this).ptr });
        let ptr = unsafe { ptr.as_ref_unchecked() };
        let ptr = ptr.load(Ordering::Relaxed);
        unsafe { ptr.as_ref_unchecked() }
    }

    pub fn read<'a>(&'a self, guard: &'a RcuGuard) -> &'a P::Target {
        let _ = guard;
        let ptr = self.ptr.get();
        let ptr = unsafe { ptr.as_ref_unchecked() };
        let ptr = ptr.load(Ordering::Relaxed);
        unsafe { ptr.as_ref_unchecked() }
    }

    pub fn read_exclusive(self: Shield<&mut Self>) -> &P::Target {
        let ptr = self.ptr.get();
        let ptr = unsafe { ptr.as_ref_unchecked() };
        let ptr = ptr.as_ptr();
        let ptr = unsafe { ptr.read() };
        unsafe { ptr.as_ref_unchecked() }
    }

    pub fn write(self: Shield<&mut Self>, new: P) -> RcuOld<P> {
        let ptr = self.ptr.get();
        let r = unsafe { ptr.as_ref_unchecked() };
        let old = r.load(Ordering::Relaxed);
        let new = P::into_raw(new);
        r.store(new, Ordering::Relaxed);
        RcuOld(unsafe { P::from_raw(old) })
    }
}

// implemented for every lock that supports `Rcu`
impl<P: RcuPointer> InsideOfMutex<Rcu<P>> {
    pub fn read<'a>(&'a self, guard: &'a RcuGuard) -> &'a P::Target {
        unsafe { Rcu::read_raw(self.0.get(), guard) }
    }
}

pub struct RcuOld<P>(P);

impl<P> Drop for RcuOld<P> {
    fn drop(&mut self) {
        unsafe { bindings::synchronize_rcu() };
    }
}

unsafe impl<T> RcuPointer for Box<T> {
    type Target = T;

    fn into_raw(this: Self) -> *mut Self::Target {
        Box::into_raw(this)
    }

    unsafe fn from_raw(raw: *mut Self::Target) -> Self {
        unsafe { Box::from_raw(raw) }
    }
}
