use std::{
    cell::{
        Cell,
        RefMut,
        SyncUnsafeCell,
        UnsafeCell,
    },
    marker::PhantomData,
};

use crate::{
    ops::place::{
        BorrowPlace,
        CreateHandle,
        PlaceHandle,
        PlaceProxy,
        PlaceWrapper,
        WrapPlace,
        borrowck::{
            AccessKind,
            Instant,
            Lifetime,
        },
    },
    place::{
        Subplace,
        TransmutedSubplace,
    },
};

impl<T: ?Sized> PlaceWrapper for Cell<T> {
    type Inner = T;
}

unsafe impl<S> WrapPlace<S> for Cell<S::Source>
where
    S: Subplace,
{
    type Wrapped = TransmutedSubplace<S, Cell<S::Source>, Cell<S::Target>>;

    fn wrap(subplace: S) -> Self::Wrapped {
        unsafe { TransmutedSubplace::new_unchecked(subplace) }
    }
}

impl<T: ?Sized> PlaceWrapper for UnsafeCell<T> {
    type Inner = T;
}

unsafe impl<S> WrapPlace<S> for UnsafeCell<S::Source>
where
    S: Subplace,
{
    type Wrapped =
        TransmutedSubplace<S, UnsafeCell<S::Source>, UnsafeCell<S::Target>>;

    fn wrap(subplace: S) -> Self::Wrapped {
        unsafe { TransmutedSubplace::new_unchecked(subplace) }
    }
}

impl<T: ?Sized> PlaceWrapper for SyncUnsafeCell<T> {
    type Inner = T;
}

unsafe impl<S> WrapPlace<S> for SyncUnsafeCell<S::Source>
where
    S: Subplace,
{
    type Wrapped = TransmutedSubplace<
        S,
        SyncUnsafeCell<S::Source>,
        SyncUnsafeCell<S::Target>,
    >;

    fn wrap(subplace: S) -> Self::Wrapped {
        unsafe { TransmutedSubplace::new_unchecked(subplace) }
    }
}

pub struct CellMutHandle<'b, T> {
    phantom: PhantomData<&'b mut T>,
}

impl<'b, T> PlaceProxy for RefMut<'b, T> {
    type Target = T;
}

unsafe impl<'b, T> CreateHandle<Instant> for RefMut<'b, T> {
    type Handle = CellMutHandle<'b, T>;

    const ACCESS: AccessKind = AccessKind::Shared;

    unsafe fn handle_from_raw(_this: *const Self) -> Self::Handle {
        CellMutHandle { phantom: PhantomData }
    }
}

impl<T> PlaceHandle for CellMutHandle<'_, T> {
    type Target = T;
}

unsafe impl<'a, 'b, T> BorrowPlace<&'a mut T> for CellMutHandle<'b, T>
where
    'b: 'a,
{
    const ACCESS: AccessKind = AccessKind::Exclusive;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> &'a mut T {
        todo!()
    }
}

unsafe impl<'a, 'b, T> BorrowPlace<RefMut<'a, T>> for CellMutHandle<'b, T>
where
    'b: 'a,
{
    const ACCESS: AccessKind = AccessKind::Exclusive;
    /// Importantly, this is `'b` and not `'a`!
    type Timing = Lifetime<'b>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> RefMut<'a, T> {
        todo!()
    }
}
