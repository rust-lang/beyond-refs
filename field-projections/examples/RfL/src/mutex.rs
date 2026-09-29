use std::{
    cell::UnsafeCell,
    ops::{
        Deref,
        DerefMut,
    },
};

use design::{
    ops::place::{
        CreateHandle,
        PlaceProxy,
        PlaceWrapper,
        WrapPlace,
        borrowck::{
            AccessKind,
            Instant,
        },
    },
    place::{
        MutHandle,
        Subplace,
        TransmutedSubplace,
    },
};

use crate::{
    bindings,
    opaque::Opaque,
    overwrite::Shield,
};

pub struct Mutex<T> {
    value: UnsafeCell<T>,
    mutex: Opaque<bindings::mutex>,
}

impl<T> Mutex<T> {
    pub fn lock(&self) -> Shield<MutexGuard<'_, T>> {
        unsafe { bindings::mutex_lock(self.mutex.get()) };
        let guard = MutexGuard(self);
        unsafe { Shield::new_unchecked(guard) }
    }

    // inaccuracy: mutex in the kernel requires pin-init, but here we simplify
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            mutex: Opaque::uninit(),
        }
    }
}

#[repr(transparent)]
pub struct InsideOfMutex<T>(pub(crate) UnsafeCell<T>);

pub struct MutexGuard<'a, T>(&'a Mutex<T>);

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { bindings::mutex_unlock(self.0.mutex.get()) };
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.value.get() }
    }
}

// auto-derived from the deref[mut] impl
impl<'a, T> PlaceProxy for MutexGuard<'a, T> {
    type Target = T;
}

unsafe impl<'a, T> CreateHandle<Instant> for MutexGuard<'a, T> {
    type Handle = MutHandle<'a, T>;

    const ACCESS: AccessKind = AccessKind::Shared;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        let ptr: *const &Mutex<T> = unsafe { &raw const (*this).0 };
        let ptr: *const *const Mutex<T> = ptr.cast();
        let ptr: *const Mutex<T> = unsafe { ptr.read() };
        let ptr: *const UnsafeCell<T> = unsafe { &raw const (*ptr).value };
        let ptr: *mut T = UnsafeCell::raw_get(ptr);
        unsafe { MutHandle::from_raw(ptr) }
    }
}

impl<T> PlaceWrapper for Mutex<T> {
    type Inner = T;
}

unsafe impl<S> WrapPlace<S> for Mutex<S::Source>
where
    S: Subplace<Source: Sized, Target: Sized>,
{
    type Wrapped =
        TransmutedSubplace<S, Mutex<S::Source>, InsideOfMutex<S::Target>>;

    fn wrap(subplace: S) -> Self::Wrapped {
        unsafe { TransmutedSubplace::new_unchecked(subplace) }
    }
}

impl<T> PlaceWrapper for InsideOfMutex<T> {
    type Inner = T;
}

unsafe impl<S> WrapPlace<S> for InsideOfMutex<S::Source>
where
    S: Subplace<Source: Sized, Target: Sized>,
{
    type Wrapped = TransmutedSubplace<
        S,
        InsideOfMutex<S::Source>,
        InsideOfMutex<S::Target>,
    >;

    fn wrap(subplace: S) -> Self::Wrapped {
        unsafe { TransmutedSubplace::new_unchecked(subplace) }
    }
}
