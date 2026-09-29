use std::{
    mem::{
        self,
        ManuallyDrop,
    },
    pin::Pin,
};

use crate::{
    ops::place::{
        BorrowPlace,
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
            Timing,
        },
    },
    place::Subplace,
    ptr::Metadata,
};

impl<P> PlaceProxy for Pin<P>
where
    P: PlaceProxy,
{
    type Target = P::Target;
}

unsafe impl<ProxyTiming, P> CreateHandle<ProxyTiming> for Pin<P>
where
    ProxyTiming: Timing,
    P: CreateHandle<ProxyTiming>,
{
    type Handle = PinnedHandle<P::Handle>;

    const ACCESS: AccessKind = P::ACCESS;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        let ptr: *const Pin<P> = this;
        let ptr: *const P = ptr.cast();
        let handle = unsafe { P::handle_from_raw(ptr) };
        PinnedHandle(handle)
    }
}

pub struct PinnedHandle<H>(H);

impl<H> PinnedHandle<H> {
    pub unsafe fn new_unchecked(handle: H) -> Self {
        Self(handle)
    }
}

impl<H> PlaceHandle for PinnedHandle<H>
where
    H: PlaceHandle,
{
    type Target = H::Target;
}

unsafe impl<H> ReadPlace for PinnedHandle<H>
where
    H: ReadPlace,
    H::Target: Sized + Unpin,
{
    const ACCESS: AccessKind = H::ACCESS;
    const SAFE: bool = H::SAFE;

    unsafe fn read_place(self) -> Self::Target {
        unsafe { self.0.read_place() }
    }
}

unsafe impl<H> ReadMetadata for PinnedHandle<H>
where
    H: ReadMetadata,
{
    fn metadata(self) -> Metadata<Self::Target> {
        self.0.metadata()
    }
}

unsafe impl<H> MovePlace for PinnedHandle<H>
where
    H: MovePlace,
    H::Target: Sized + Unpin,
{
}

unsafe impl<H> WritePlace for PinnedHandle<H>
where
    H: WritePlace,
    H::Target: Sized,
{
    const ACCESS: AccessKind = H::ACCESS;
    const SAFE: bool = H::SAFE;

    unsafe fn write_place(self, value: Self::Target) {
        unsafe { self.0.write_place(value) }
    }
}

unsafe impl<H> DropPlace for PinnedHandle<H>
where
    H: DropPlace,
{
    unsafe fn drop_place(self) {
        unsafe { self.0.drop_place() };
    }
}

unsafe impl<P> DropHusk for Pin<P>
where
    P: DropHusk,
{
    unsafe fn drop_husk(this: *mut Self) {
        // The entire pointee is dropped, so no pin guarantee is left.
        unsafe { P::drop_husk(this.cast()) }
    }
}

pub unsafe trait PinnableSubplace: Subplace {
    /// The structural pinnedness of this subplace.
    ///
    /// Must be exactly one of these two GATs:
    /// - `H` (i.e. the identity GAT)
    /// - `PinnedHandle<H>`
    type StructualPinning<H: PlaceHandle<Target = Self::Target>>: PlaceHandle<
        Target = Self::Target,
    >;

    /// - If `StructualPinning<H> == H`, then this should be the identity
    ///   function.
    /// - If `StructualPinning<H> == PinnedHandle<H>`, then this should be
    ///   `PinnedHandle::new_unchecked`.
    unsafe fn from_pinned<H: PlaceHandle<Target = Self::Target>>(
        handle: H,
    ) -> Self::StructualPinning<H>;
}

unsafe impl<H, S> ProjectPlace<S> for PinnedHandle<H>
where
    H: ProjectPlace<S>,
    S: PinnableSubplace<Source = H::Target>,
{
    type Projected = S::StructualPinning<H::Projected>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let handle = unsafe { self.0.project_place(subplace) };
        unsafe { S::from_pinned(handle) }
    }
}

// FIXME: we probably need something specific to pin here to, but haven't given
// it much thought...
unsafe impl<H, PointeeTiming, PointerTiming>
    DerefPlace<PointeeTiming, PointerTiming> for PinnedHandle<H>
where
    Self::Target: CreateHandle<PointeeTiming>,
    H: DerefPlace<PointeeTiming, PointerTiming>,
    PointeeTiming: Timing,
    PointerTiming: Timing,
{
    const POINTEE_ACCESS: AccessKind = H::POINTEE_ACCESS;
    const POINTER_ACCESS: AccessKind = H::POINTER_ACCESS;
    const SAFE: bool = H::SAFE;

    unsafe fn deref_place(
        self,
    ) -> <Self::Target as CreateHandle<PointeeTiming>>::Handle {
        let handle = unsafe { self.0.deref_place() };
        handle
    }
}

unsafe impl<H, Output> BorrowPlace<Pin<Output>> for PinnedHandle<H>
where
    H: BorrowPlace<Output>,
{
    const ACCESS: AccessKind = H::ACCESS;
    type Timing = H::Timing;
    const SAFE: bool = H::SAFE;

    unsafe fn borrow(self) -> Pin<Output> {
        unsafe { new_pin_unchecked(self.0.borrow()) }
    }
}

unsafe fn new_pin_unchecked<T>(t: T) -> Pin<T> {
    let t = ManuallyDrop::new(t);
    // somehow can't use `mem::transmute`?
    unsafe { mem::transmute_copy::<T, Pin<T>>(&t) }
}
