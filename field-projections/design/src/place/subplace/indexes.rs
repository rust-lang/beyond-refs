use std::marker::PhantomCovariant;

use crate::{
    place::Subplace,
    ptr::Metadata,
};

pub struct ArrayIndex<T, const LEN: usize>(usize, PhantomCovariant<T>);

impl<T, const LEN: usize> ArrayIndex<T, LEN> {
    pub unsafe fn new_unchecked(idx: usize) -> Self {
        debug_assert!(idx < LEN);
        Self(idx, PhantomCovariant::new())
    }
}

unsafe impl<T, const LEN: usize> Subplace for ArrayIndex<T, LEN> {
    type Source = [T; LEN];
    type Target = T;

    fn offset(
        self,
        (): Metadata<Self::Source>,
    ) -> (usize, Metadata<Self::Target>) {
        const { assert!(size_of::<T>().strict_mul(LEN) < isize::MAX as usize) }
        debug_assert!(self.0 < LEN);
        (self.0 * size_of::<T>(), ())
    }
}

pub struct SliceIndex<T>(usize, PhantomCovariant<T>);

unsafe impl<T> Subplace for SliceIndex<T> {
    type Source = [T];
    type Target = T;

    fn offset(
        self,
        len: Metadata<Self::Source>,
    ) -> (usize, Metadata<Self::Target>) {
        debug_assert!(len * size_of::<T>() < isize::MAX as usize);
        (self.0 * size_of::<T>(), ())
    }
}
