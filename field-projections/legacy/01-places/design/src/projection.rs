use std::{
    marker::PhantomData,
    ptr::Pointee,
};

use crate::*;

// TODO: inspect projections
pub trait Projection {
    type Source: ?Sized;
    type Target: ?Sized;

    fn offset(&self, meta: <Self::Source as Pointee>::Metadata) -> usize;
    fn project_metadata(
        &self,
        meta: <Self::Source as Pointee>::Metadata,
    ) -> <Self::Target as Pointee>::Metadata;
}

/// Extension trait so that `Projection` stays dyn-compatible.
impl<P: Projection + ?Sized> ProjectionExt for P {}
pub trait ProjectionExt: Projection {
    /// Convenience method that simply calls the corresponding PlaceBorrow
    /// method.
    unsafe fn borrow<'a, X, Y>(&self, ptr: *const X) -> Y
    where
        X: HasPlace<Target = Self::Source>,
        Y: HasPlace<Target = Self::Target>,
        X: PlaceBorrow<'a, Self, Y>,
    {
        unsafe { PlaceBorrow::borrow(ptr, self) }
    }
    /// Convenience method that simply calls the corresponding PlaceRead method.
    unsafe fn read<X>(&self, ptr: *const X) -> Self::Target
    where
        X: PlaceRead<Self>,
        Self::Target: Sized,
    {
        unsafe { PlaceRead::read(ptr, self) }
    }
    /// Convenience method that simply calls the corresponding PlaceRead method.
    unsafe fn write<X>(&self, ptr: *mut X, val: Self::Target)
    where
        X: PlaceWrite<Self>,
        Self::Target: Sized,
    {
        unsafe { PlaceWrite::write(ptr, self, val) }
    }
    /// Convenience method that simply calls the corresponding PlaceDeref
    /// method.
    unsafe fn deref<X>(&self, ptr: *mut X) -> *const Self::Target
    where
        X: HasPlace<Target = Self::Source>,
        X: PlaceDeref<Self>,
        Self::Target: HasPlace,
    {
        unsafe { PlaceDeref::double_deref(ptr, self) }
    }

    /// When the target is sized, we know a projection is just an offset so we
    /// can make it sized even if we had a `dyn Projection`.
    ///
    /// Definitely a bit hacky.
    fn as_sized(&self) -> SizedProj<Self::Source, Self::Target>
    where
        Self::Source: Sized,
        Self::Target: Sized,
    {
        SizedProj(self.offset(()), PhantomData, PhantomData)
    }

    fn compose<Q>(self, other: Q) -> ComposeProj<Self, Q>
    where
        Self: Sized,
        Q: Projection<Source = Self::Target>,
    {
        ComposeProj { p: self, q: other }
    }
}

pub struct NoopProj<T: ?Sized>(PhantomData<T>);
impl<T> Default for NoopProj<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<T> Clone for NoopProj<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: ?Sized> Projection for NoopProj<T> {
    type Source = T;
    type Target = T;
    fn offset(&self, _m: <Self::Source as Pointee>::Metadata) -> usize {
        0
    }
    fn project_metadata(
        &self,
        m: <Self::Source as Pointee>::Metadata,
    ) -> <Self::Target as Pointee>::Metadata {
        m
    }
}

/// Sized projection that holds only an offset.
#[derive(Clone)]
pub struct SizedProj<S: ?Sized, T>(usize, PhantomData<S>, PhantomData<T>);
impl<S, T> Projection for SizedProj<S, T> {
    type Source = S;
    type Target = T;
    fn offset(&self, _m: <Self::Source as Pointee>::Metadata) -> usize {
        self.0
    }
    fn project_metadata(
        &self,
        _: <Self::Source as Pointee>::Metadata,
    ) -> <Self::Target as Pointee>::Metadata {
    }
}

/// Projection `P` followed by `Q`. `P` may be unsized.
#[derive(Clone)]
pub struct ComposeProj<P: ?Sized, Q> {
    q: Q,
    p: P,
}
impl<P, Q> Projection for ComposeProj<P, Q>
where
    P: Projection + ?Sized,
    Q: Projection<Source = P::Target>,
{
    type Source = P::Source;
    type Target = Q::Target;
    fn offset(&self, meta: <Self::Source as Pointee>::Metadata) -> usize {
        let qmeta = self.p.project_metadata(meta);
        self.p.offset(meta) + self.q.offset(qmeta)
    }
    fn project_metadata(
        &self,
        meta: <Self::Source as Pointee>::Metadata,
    ) -> <Self::Target as Pointee>::Metadata {
        self.q.project_metadata(self.p.project_metadata(meta))
    }
}
