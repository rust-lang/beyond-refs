use std::ops::{
    Deref,
    DerefMut,
};

use design::{
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

pub auto trait Overwrite {}

pub struct Shield<P>(P);
impl<P> Shield<P> {
    pub unsafe fn new_unchecked(pointer: P) -> Self {
        Self(pointer)
    }

    pub fn as_mut(&mut self) -> Shield<&mut P::Target>
    where
        P: DerefMut,
    {
        Shield(self.0.deref_mut())
    }

    pub unsafe fn into_inner_unchecked(this: Self) -> P {
        this.0
    }
}

impl<P> Deref for Shield<P>
where
    P: Deref,
{
    type Target = P::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<P> DerefMut for Shield<P>
where
    P: DerefMut,
    P::Target: Overwrite,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

pub struct ShieldHandle<H>(H);

impl<H> ShieldHandle<H> {
    pub unsafe fn new_unchecked(handle: H) -> Self {
        Self(handle)
    }
}

impl<H: PlaceHandle> PlaceHandle for ShieldHandle<H> {
    type Target = H::Target;
}
impl<P> PlaceProxy for Shield<P>
where
    P: PlaceProxy,
{
    type Target = P::Target;
}

unsafe impl<P, ProxyTiming> CreateHandle<ProxyTiming> for Shield<P>
where
    P: CreateHandle<ProxyTiming>,
    ProxyTiming: Timing,
{
    type Handle = ShieldHandle<P::Handle>;

    const ACCESS: AccessKind = P::ACCESS;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        let ptr: *const Shield<P> = this;
        let ptr: *const P = ptr.cast();
        let handle = unsafe { P::handle_from_raw(ptr) };
        ShieldHandle(handle)
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

unsafe impl<H> ReadPlace for ShieldHandle<H>
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

unsafe impl<H> ReadMetadata for ShieldHandle<H>
where
    H: ReadMetadata,
{
    fn metadata(self) -> Metadata<Self::Target> {
        self.0.metadata()
    }
}

unsafe impl<H> MovePlace for ShieldHandle<H>
where
    H: MovePlace,
    H::Target: Sized + Unpin,
{
}

unsafe impl<H> WritePlace for ShieldHandle<H>
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

unsafe impl<H> DropPlace for ShieldHandle<H>
where
    H: DropPlace,
{
    unsafe fn drop_place(self) {
        unsafe { self.0.drop_place() };
    }
}

unsafe impl<P> DropHusk for Shield<P>
where
    P: DropHusk,
{
    unsafe fn drop_husk(this: *mut Self) {
        unsafe { P::drop_husk(&raw mut (*this).0) }
    }
}

pub unsafe trait ShieldableSubplace: Subplace {
    /// The structural shieldedness of this subplace.
    ///
    /// Must be exactly one of these two GATs:
    /// - `H` (i.e. the identity GAT)
    /// - `ShieldHandle<H>`
    type StructualShielding<H: PlaceHandle<Target = Self::Target>>: PlaceHandle<
        Target = Self::Target,
    >;

    /// - If `StructualShielding<H> == H`, then this should be the identity
    ///   function.
    /// - If `StructualShielding<H> == ShieldHandle<H>`, then this should be
    ///   `ShieldHandle::new_unchecked`.
    unsafe fn from_shielded<H: PlaceHandle<Target = Self::Target>>(
        handle: H,
    ) -> Self::StructualShielding<H>;
}

unsafe impl<S: Subplace> ShieldableSubplace for S
where
    S::Target: Overwrite,
{
    type StructualShielding<H: PlaceHandle<Target = Self::Target>> = H;

    unsafe fn from_shielded<H: PlaceHandle<Target = Self::Target>>(
        handle: H,
    ) -> Self::StructualShielding<H> {
        handle
    }
}

/*
 * Would like to write this impl, but can't because negative reasoning doesn't exist.
 *
impl<S: Subplace> ShieldableSubplace for S
where
    S::Target: !Overwrite,
{
    type StructualShielding<H: PlaceHandle<Target = Self::Target>> = ShieldHandle<H>;

    unsafe fn from_shielded<H: PlaceHandle<Target = Self::Target>>(
        handle: H,
    ) -> Self::StructualShielding<H> {
        ShieldHandle(handle)
    }
}
*/

unsafe impl<H, S> ProjectPlace<S> for ShieldHandle<H>
where
    H: ProjectPlace<S>,
    S: ShieldableSubplace<Source = H::Target>,
{
    type Projected = S::StructualShielding<H::Projected>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let handle = unsafe { self.0.project_place(subplace) };
        unsafe { S::from_shielded(handle) }
    }
}

unsafe impl<H, PointeeTiming, PointerTiming>
    DerefPlace<PointeeTiming, PointerTiming> for ShieldHandle<H>
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

unsafe impl<H, Output> BorrowPlace<Shield<Output>> for ShieldHandle<H>
where
    H: BorrowPlace<Output>,
{
    const ACCESS: AccessKind = H::ACCESS;
    type Timing = H::Timing;
    const SAFE: bool = H::SAFE;

    unsafe fn borrow(self) -> Shield<Output> {
        unsafe { Shield::new_unchecked(self.0.borrow()) }
    }
}
