use crate::*;

pub trait HasPlace {
    type Target: ?Sized;
}

/// Borrow a subplace.
pub unsafe trait PlaceBorrow<'a, P, X>
where
    P: Projection + ?Sized,
    Self: HasPlace<Target = P::Source>,
    X: HasPlace<Target = P::Target>,
{
    /// Tells the borrow-checker what other simultaneous and subsequent
    /// borrows are allowed.
    const BORROW_KIND: BorrowKind;

    unsafe fn borrow(ptr: *const Self, p: &P) -> X;
}

#[non_exhaustive]
pub enum BorrowKind {
    /// Other borrows are allowed (like `*mut T` and `RcRef<T>`).
    Untracked,
    /// Other `Shared` simultaneous borrows are allowed (like `&T`).
    Shared,
    /// No other simultaneous tracked borrows are allowed (like `&mut T`).
    Unique,
    /// No other simultaneous or subsequent borrows are allowed (like `&own T`).
    Owning,
    /// No other simultaneous tracked borrows are allowed and `drop` must be
    /// called before the underlying memory is reclaimed (like `&pin mut T`).
    UniquePinning,
    // maybe other things?
}

/// Read a value from a subplace.
pub unsafe trait PlaceRead<P>
where
    P: Projection + ?Sized,
    Self: HasPlace<Target = P::Source>,
{
    unsafe fn read(ptr: *const Self, p: &P) -> P::Target
    where
        P::Target: Sized;
}

/// Write to a subplace.
pub unsafe trait PlaceWrite<P>
where
    P: Projection + ?Sized,
    Self: HasPlace<Target = P::Source>,
{
    unsafe fn write(ptr: *mut Self, p: &P, x: P::Target)
    where
        P::Target: Sized;
}

/// Allows moving a value out of a subplace. This uses `PlaceRead::read` to read
/// the value.
pub unsafe trait PlaceMove<P>: PlaceRead<P>
where
    P: Projection + ?Sized,
{
}

/// Allows dereferencing a subplace that contains a pointer. The returned
/// pointer will only be used for further place operations.
pub unsafe trait PlaceDeref<P>
where
    P: Projection + ?Sized,
    P::Target: HasPlace,
    Self: HasPlace<Target = P::Source>,
{
    unsafe fn double_deref(ptr: *mut Self, p: &P) -> *const P::Target;
}

/// Drop the contents of a subplace.
pub unsafe trait PlaceDrop<P>
where
    P: Projection + ?Sized,
    P::Target: HasPlace,
    Self: HasPlace<Target = P::Source>,
{
    /// Should call `drop_in_place` on the subplace.
    unsafe fn drop(ptr: *mut Self, p: &P);
}

/// Clean up a pointer whose contained place has been moved out of/dropped.
pub unsafe trait DropHusk: HasPlace {
    /// Drop the pointer but not the contents of the place (borrowck takes care
    /// of that).
    unsafe fn drop_husk(ptr: *mut Self);
}

/// If at a coercion site an expression `e` has type `T` but type `U` was
/// expected, and `T: HasPlace` and `T::Target: PlaceCoerce<T>`, then we replace
/// `e` with `@T::Target::Output **e` and repeat this until types match and
/// raise an error otherwise.
pub unsafe trait PlaceCoerce<From>: HasPlace
where
    From: HasPlace<Target = Self>,
    From: PlaceDeref<NoopProj<Self>>,
{
    type Output: HasPlace<Target = Self::Target>;
}

/// If `X: PlaceWrap` and `X::Target` has a field `field`, then `X` itself
/// acquires a virtual field named `field` as well. That field has type `<X as
/// PlaceWrap<proj_ty!(X::Target.field)>>::WrappedProj::Target`, and
/// `WrappedProj` is the projection used when we refer to that field.
pub unsafe trait PlaceWrap<P: Projection<Source = Self::Target>>:
    HasPlace
{
    type WrappedProj: Projection<Source = Self>;
    fn wrap_proj(p: &P) -> Self::WrappedProj;
}
