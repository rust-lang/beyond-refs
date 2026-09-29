use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
};

pub struct Opaque<T>(UnsafeCell<MaybeUninit<T>>);

impl<T> Opaque<T> {
    pub fn get(&self) -> *mut T {
        self.0.get().cast()
    }

    pub fn uninit() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }
}
