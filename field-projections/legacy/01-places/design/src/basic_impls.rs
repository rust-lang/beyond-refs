use crate::*;

// Aliases to make macros and copy/paste easier.
type RawConst<T> = *const T;
type RawMut<T> = *mut T;
type SharedRef<'a, T> = &'a T;
type MutRef<'a, T> = &'a mut T;

macro_rules! impl_has_place {
    ($ptr:ident) => {
        impl<T: ?Sized> HasPlace for $ptr<T> {
            type Target = T;
        }
    };
}
macro_rules! impl_has_place_with_lt {
    ($ptr:ident) => {
        impl<'a, T: ?Sized> HasPlace for $ptr<'a, T> {
            type Target = T;
        }
    };
}

impl_has_place!(RawConst);
impl_has_place!(RawMut);
impl_has_place!(NonNull);
impl_has_place_with_lt!(SharedRef);
impl_has_place_with_lt!(MutRef);

// The two basic impls everything else is derived from.
unsafe impl<'a, P: Projection + ?Sized> PlaceBorrow<'a, P, RawConst<P::Target>>
    for RawConst<P::Source>
{
    const BORROW_KIND: BorrowKind = BorrowKind::Untracked;
    unsafe fn borrow(ptr: *const Self, p: &P) -> RawConst<P::Target> {
        unsafe {
            let (ptr, meta) = (*ptr).to_raw_parts();
            let ptr = ptr.byte_offset(p.offset(meta) as isize);
            let meta = p.project_metadata(meta);
            std::ptr::from_raw_parts(ptr, meta)
        }
    }
}
unsafe impl<'a, P: Projection + ?Sized> PlaceBorrow<'a, P, RawMut<P::Target>>
    for RawMut<P::Source>
{
    const BORROW_KIND: BorrowKind = BorrowKind::Untracked;
    unsafe fn borrow(ptr: *const Self, p: &P) -> RawMut<P::Target> {
        unsafe {
            let (ptr, meta) = (*ptr).to_raw_parts();
            let ptr = ptr.byte_offset(p.offset(meta) as isize);
            let meta = p.project_metadata(meta);
            std::ptr::from_raw_parts_mut(ptr, meta)
        }
    }
}

unsafe impl<'a, P: Projection + ?Sized> PlaceBorrow<'a, P, NonNull<P::Target>>
    for NonNull<P::Source>
{
    const BORROW_KIND: BorrowKind = BorrowKind::Untracked;
    unsafe fn borrow(ptr: *const Self, p: &P) -> NonNull<P::Target> {
        unsafe {
            let ptr: *mut _ = (*ptr).as_ptr();
            NonNull::new_unchecked(p.borrow::<*mut _, *mut _>(&raw const ptr))
        }
    }
}
unsafe impl<'a, P: Projection + ?Sized> PlaceBorrow<'a, P, RawConst<P::Target>>
    for NonNull<P::Source>
{
    const BORROW_KIND: BorrowKind = BorrowKind::Untracked;
    unsafe fn borrow(ptr: *const Self, p: &P) -> RawConst<P::Target> {
        unsafe { p.borrow::<NonNull<_>, NonNull<_>>(ptr).as_ptr() }
    }
}
unsafe impl<'a, P: Projection + ?Sized> PlaceBorrow<'a, P, RawConst<P::Target>>
    for RawMut<P::Source>
{
    const BORROW_KIND: BorrowKind = BorrowKind::Untracked;
    unsafe fn borrow(ptr: *const Self, p: &P) -> RawConst<P::Target> {
        unsafe { p.borrow::<RawMut<_>, RawMut<_>>(ptr) }
    }
}
unsafe impl<'a, P: Projection + ?Sized> PlaceBorrow<'a, P, RawConst<P::Target>>
    for SharedRef<'_, P::Source>
{
    const BORROW_KIND: BorrowKind = BorrowKind::Untracked;
    unsafe fn borrow(ptr: *const Self, p: &P) -> RawConst<P::Target> {
        unsafe {
            let r: *const *const Self::Target = ptr.cast();
            p.borrow::<RawConst<_>, RawConst<_>>(r)
        }
    }
}
unsafe impl<'a, P: Projection + ?Sized> PlaceBorrow<'a, P, RawConst<P::Target>>
    for MutRef<'_, P::Source>
{
    const BORROW_KIND: BorrowKind = BorrowKind::Untracked;
    unsafe fn borrow(ptr: *const Self, p: &P) -> RawConst<P::Target> {
        unsafe {
            let r: *const *mut Self::Target = ptr.cast();
            p.borrow::<RawMut<_>, RawMut<_>>(r)
        }
    }
}

unsafe impl<P: Projection + ?Sized> PlaceDeref<P> for NonNull<P::Source>
where
    P::Target: HasPlace,
{
    unsafe fn double_deref(
        ptr: *mut Self,
        p: &P,
    ) -> *const <P as Projection>::Target {
        unsafe { p.borrow(ptr) }
    }
}
unsafe impl<P: Projection + ?Sized> PlaceDeref<P> for RawConst<P::Source>
where
    P::Target: HasPlace,
{
    unsafe fn double_deref(
        ptr: *mut Self,
        p: &P,
    ) -> *const <P as Projection>::Target {
        unsafe { p.borrow(ptr) }
    }
}
unsafe impl<P: Projection + ?Sized> PlaceDeref<P> for RawMut<P::Source>
where
    P::Target: HasPlace,
{
    unsafe fn double_deref(
        ptr: *mut Self,
        p: &P,
    ) -> *const <P as Projection>::Target {
        unsafe { p.borrow(ptr) }
    }
}
unsafe impl<P: Projection + ?Sized> PlaceDeref<P> for SharedRef<'_, P::Source>
where
    P::Target: HasPlace,
{
    unsafe fn double_deref(
        ptr: *mut Self,
        p: &P,
    ) -> *const <P as Projection>::Target {
        unsafe { p.borrow(ptr) }
    }
}
unsafe impl<P: Projection + ?Sized> PlaceDeref<P> for MutRef<'_, P::Source>
where
    P::Target: HasPlace,
{
    unsafe fn double_deref(
        ptr: *mut Self,
        p: &P,
    ) -> *const <P as Projection>::Target {
        unsafe { p.borrow(ptr) }
    }
}

unsafe impl<P: Projection + ?Sized> PlaceRead<P> for RawConst<P::Source> {
    unsafe fn read(ptr: *const Self, p: &P) -> P::Target
    where
        P::Target: Sized,
    {
        unsafe { p.borrow::<RawConst<_>, RawConst<_>>(ptr).read() }
    }
}
unsafe impl<P: Projection + ?Sized> PlaceRead<P> for RawMut<P::Source> {
    unsafe fn read(ptr: *const Self, p: &P) -> P::Target
    where
        P::Target: Sized,
    {
        // Use read on `*const`.
        unsafe { p.read(ptr.cast::<*const _>()) }
    }
}
unsafe impl<P: Projection + ?Sized> PlaceRead<P> for SharedRef<'_, P::Source> {
    unsafe fn read(ptr: *const Self, p: &P) -> P::Target
    where
        P::Target: Sized,
    {
        // Use read on `*const`.
        unsafe { p.read(ptr.cast::<*const _>()) }
    }
}

unsafe impl<P: Projection + ?Sized> PlaceWrite<P> for RawMut<P::Source> {
    unsafe fn write(ptr: *mut Self, p: &P, x: P::Target)
    where
        P::Target: Sized,
    {
        unsafe { p.borrow::<RawMut<_>, RawMut<_>>(ptr).write(x) }
    }
}

unsafe impl<'a, 'b, T: ?Sized> PlaceCoerce<&'a &'b T> for &'b T {
    type Output = &'b T;
}
unsafe impl<'a, 'b, T: ?Sized> PlaceCoerce<&'a mut &'b T> for &'b T {
    type Output = &'b T;
}
unsafe impl<'a, 'b, T: ?Sized> PlaceCoerce<&'a &'b mut T> for &'b mut T {
    type Output = &'a T;
}
unsafe impl<'a, 'b, T: ?Sized> PlaceCoerce<&'a mut &'b mut T> for &'b mut T {
    type Output = &'a mut T;
}
