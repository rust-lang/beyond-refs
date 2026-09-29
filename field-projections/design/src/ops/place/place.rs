//! ## Operations on places
//!
//! <details><summary>Birds-eye view of this module</summary>
//!
//! Here are all the items defined by this module without any attributes or
//! other distractions:
//!
//! ```ignore
#![doc = macros::raw_summary!()]
//! ```
//!
//! </details>
//!
//! This module provides traits to customize the place operations of Rust.
//!
//! ## Places and Place Expressions
//!
//! A *place* in Rust is a particular location in memory. Places are represented
//! by [*place expressions*][ref-place-exprs], which have the following syntax:
//!
//! [ref-place-exprs]: https://doc.rust-lang.org/reference/expressions.html#r-expr.place-value.place-expr-kinds
//!
//! - `$path`: paths that refer to local variables (also parameters) and
//!   statics,
//! - `*$place`: dereferencing another place expression,
//! - `$place[$expr]`: indexing into another place expression,
//! - `$place.$ident`: accessing a field of another place expression,
//! - `($place)`: parenthesized place expressions,
//! - `$value`: an arbitrary expression; it's value is stored in a temporary
//!   place whose lifetime is determined from its context.
//!
//! Further reading:
//! - <https://nadrieril.github.io/blog/2025/12/06/on-places-and-their-magic.html>
//! - <https://www.ralfj.de/blog/2024/08/14/places.html>
//!
//! Place expressions have a direct representation in the form of a
//! [`PlaceHandle`]. A handle points at a place and is responsible for
//! performing all available place operations on the represented place.
//! Any valid place expression is converted into a handle by the compiler. We
//! represent this conversion with the `handle!` pseudo-macro, explained below.
//!
//! The information a handle encodes can be broken into three primary categories:
//!
//! - The type of the place the handle refers to.
//! - Runtime information needed to access the place; usually, a pointer.
//! - The capabilities of the pointer the handle was derived from.
//!
//! For example:
//!
//! ```ignore
//! let a: &mut Struct;
//! let b: MutHandle<'b, Field> = handle!((*a).field);
//! ```
//!
//! Semantically, we can tell from the type of the handle that this place
//! expression is derived from a mutable reference, that the place is of type
//! `Field`, and that the handle has permission to access the place for the
//! lifetime `'b`.
//!
//! The definition of [`MutHandle<'a, T>`](crate::place::MutHandle) itself is
//! simple: It stores a pointer to the place (the field, in the example above).
//! It also embeds a `PhantomData<&'a mut T>` to mark the type `T` as invariant.
//!
//! To see how a place handle represents capabilities, observe how we turn this
//! expression back into a mutable reference of the field:
//!
//! ```no_run
//! # use design::{place::*, ops::place::*};
//! # struct Struct { field: Field }; struct Field;
//! # let b: MutHandle<Field> = todo!();
//! # unsafe {
//! let c: &mut Field = <MutHandle<'_, Field> as BorrowPlace<&mut Field>>::borrow(b);
//! # }
//! ```
//!
//! If the handle had been derived from a shared reference (`a` was `&Struct`
//! above), the handle type would instead be `RefHandle<'_, Field>` which does
//! not implement `BorrowPlace<&mut Field>`. It does implement
//! `BorrowPlace<&Field>`, so the following code compiles.
//!
//! ```no_run
//! # use design::{place::*, ops::place::*};
//! # struct Struct { field: Field }; struct Field;
//! # #[cfg(false)]
//! let a: &mut Struct;
//! # #[cfg(false)]
//! let b: RefHandle<'b, Field> = handle!((*a).field);
//! # let b: RefHandle<Field> = todo!();
//! # unsafe {
//! let c: &Field = <RefHandle<'_, Field> as BorrowPlace<&Field>>::borrow(b);
//! # }
//! ```
//!
//! ### Converting place expressions into handles
//!
//! Any valid place expression is converted into a handle by the compiler. We
//! give this desugaring as a pseudo[^1]-macro definition:
//!
//! [^1]: we use the non-existent macro fragment specifiers of `place` for place
//!       expressions and `member` for struct field names and tuple indices.
//!       Additionally, we require eager expansion of macros, as we use the
//!       output of the `handle!` macro in the input of the `subplace!` macro.
//!
//! ```ignore
//! macro_rules! handle {
//!     ($path:path) => { LocalHandle::new(&raw {const,mut} $path) };
//!
//!     ($place:place[$expr:expr]) => {
//!         <
//!             typeof($place) as IndexPlace<typeof($expr), _>
//!         >::index(handle!($place), $expr)
//!     };
//!
//!     (*$place:place) => { DerefPlace::deref_place(handle!($place)) };
//!
//!     ($place:place.$field:member) => {
//!         let subplace
//!             = subplace!(typeof($place), $field, handle!($place));
//!         ProjectPlace::<typeof(subplace)>::project_place(
//!             handle!($place),
//!             subplace,
//!         )
//!     };
//!
//!     (($place:place)) => { handle!($place) };
//!
//!     // value-to-place coercions are turned into temporaries:
//!     ($value:expr) => {{
//!         super let value = $value;
//!         handle!(value)
//!     }};
//! }
//! ```
//!
//! Links to the types and traits used: [`LocalHandle`], [`IndexPlace`],
//! [`DerefPlace`], and [`ProjectPlace`].
//!
//! [`LocalHandle`]: crate::place::LocalHandle
//!
//! The need for a `subplace!` pseudo-macro might seem surprising, but it's
//! required to support *place wrappers*, which we will cover in the next
//! section. Without place wrappers, we could simply replace the `subplace!`
//! invocation with
//! <code><[field_of!](std::field::field_of)(typeof($place), $field)>::[default](Default::default)()</code>.
//!
//! ### Place Wrappers
//!
//! Place wrappers are a special kind of place proxy. They "physically contain"
//! the place they are proxying for. A good example is [`MaybeUninit<T>`]. To
//! support subplaces of these place wrappers, the [`WrapPlace`] trait exists.
//! It allows forwarding subplaces to the proxy and changing the subplace access
//! information. With [`MaybeUninit<T>`], this allows accessing any subplace
//! under the transformation that its type is wrapped in `MaybeUninit`. So
//! given `&MaybeUninit<Struct>`, the `field` subplace can be borrowed using `&`
//! and it has type `&MaybeUninit<Field>`.
//!
//! [`MaybeUninit<T>`]: std::mem::MaybeUninit
//!
//! ## Place Operation Traits
//!
//! Aside from the three basic place operations of [`ReadPlace`],
//! [`WritePlace`], and [`BorrowPlace`], there are also the following other
//! operations:
//!
//! - [`MovePlace`] -- moving out of a place (looks the same as a read
//!   operation).
//! - [`IndexPlace`] -- using the index operator on places (`place[idx]`).
//! - [`DerefPlace`] -- dereferencing a pointer that's in a place (`*place`).
//! - [`ProjectPlace`] -- accessing a subplace (`place.field`).
//! - [`DropPlace`] -- dropping the contents of a place (no surface syntax,
//!   emitted by the compiler).
//! - [`ReadMetadata`] -- TODO (no surface syntax, emitted by the compiler).
//! - [`ReadVariant`] -- TODO (no surface syntax, emitted by the compiler).
//! - [`VariantPlace`] -- TODO (no surface syntax, emitted by the compiler).
//!
//! The place operations are desugared in a similar manner to place expressions.
//! In particular, a place operation always operates upon a place expression,
//! which relies on the desugaring detailed in the previous section. Refer to
//! the operation traits for their desugaring.
//!
//! ## Safety
//!
//! All operation functions are `unsafe`, since they have raw pointer arguments
//! that have safety preconditions. The arguments are raw pointers, because the
//! values they point to need not be in a valid state (they may be partially
//! moved out or borrowed).
//!
//! The safety requirements for the operation functions have not been figured
//! out at this point in time. Since we expect several changes to the design, we
//! do not want to commit to writing down good safety documentation before
//! having finished the design.
//!
//! What is clear at the moment is that the safety requirements will heavily
//! interact with the borrow checker. It will ensure that simultaneous place
//! operations on the same value are allowed, since they either affect disjoint
//! subplaces, or because they both only require shared access. For example:
//! - reading `ptr.field.subfield` and borrowing `ptr.field` with `&T` are
//!   allowed to happen at the same time,
//! - writing `ptr.field` and borrowing `ptr.field.subfield` at the same time is
//!   not allowed.
//!
//! The safety of using the place operations via the operators will depend on
//! the value of the `SAFE` constant in the operation traits. At the moment we
//! will only permit a literal value of `true` or `false` in implementations. It
//! will dictate if people have to write for example `unsafe { &*ptr }` or if
//! `*ptr` is allowed. It should be set to `true` when the borrow checker's
//! guarantees of either disjoint subplaces or "all concurrent operations are
//! shared" are enough to calling the operations' function correctly. If there
//! are additional requirements, such as "ptr is valid", then `SAFE` should be
//! set to `false`. For example, `&mut T` will have `SAFE = true` in
//! [`ReadPlace`], but `NonNull<T>` will set it to `false`.
//!
//! ### Handle Validity
//!
//! TODO

#[macros::summary(skip)]
use crate::{
    ops::place::borrowck::{
        AccessKind,
        Timing,
    },
    place::{
        HasVariant,
        Matchable,
        Subplace,
        VariantType,
    },
    ptr::Metadata,
};

#[macros::summary(skip)]
pub mod borrowck;

/// A type proxying for a place.
///
/// A value of this type represents a specific place. The operations that are
/// available on that place are controlled by which *place operation traits* are
/// implemented on the handle of that place, which is [`CreateHandle::Handle`].
pub trait PlaceProxy {
    type Target: ?Sized;
}

/// TODO: Defines creation of handles.
///
/// A handle to the represented place can be obtained from a value of this type
/// by calling <code>Self::[handle_from_raw]</code>.
///
/// The timing of the access permissions of [`Self::handle_from_raw`] is
/// `ProxyTiming`.
///
/// The <code>Self::[ACCESS]</code> constant specifies what type of permission
/// is required for creating a handle this way. The `ProxyTiming` argument
/// specifies for how long that permission must be granted; it must be one of
/// the types in the [`borrowck`] module. Any compiler-generated handle
/// creations automatically honor these requirements via the borrow checker.
///
/// [`borrowck`]: crate::ops::place::borrowck
/// [ACCESS]: Self::ACCESS
/// [handle_from_raw]: Self::handle_from_raw
pub unsafe trait CreateHandle<ProxyTiming: Timing>: PlaceProxy {
    /// The *handle* that's used for operating on the represented place.
    ///
    /// This type controls which place operations are available on the
    /// represented place. For example, if this implements [`ReadPlace`], then
    /// writing `*self` is allowed and yields a value of type
    /// <code>Self::[Handle]::[Target](PlaceHandle::Target)</code> (where `self:
    /// Self`).
    ///
    ///
    /// [Handle]: Self::Handle
    /// [ACCESS]: Self::ACCESS
    /// [handle_from_raw]: Self::handle_from_raw
    type Handle: PlaceHandle<Target = Self::Target>;

    /// The access permissions required by [`Self::handle_from_raw`].
    const ACCESS: AccessKind;

    /// Create a handle to the pointee of the raw pointer.
    ///
    /// # Safety
    ///
    /// - `this` must be a valid pointer for as long as the return value lives,
    /// - `*this` must be [handle-valid] with permissions [`Self::ACCESS`] for
    ///   `ProxyTiming`.
    ///
    /// [handle-valid]: self#handle-validity
    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle;
}

/// A *handle* to a place.
///
/// All place operations are carried out by handles. Whether a place operation
/// is available depends on the handle type implementing the respective [*place
/// operation trait*][self#place-operation-traits].
///
/// Place handles are not user-facing. Instead, they and the use of their
/// operations are emitted by the compiler as part of desugaring [*place
/// expressions*](self#places-and-place-expressions) and place operations.
pub trait PlaceHandle: Sized {
    /// The type that's stored in the place.
    type Target: ?Sized;
}

/// `let _ = place;` -- Read from a place.
///
/// Reading from a place doesn't have any special syntax, instead a read can
/// happens in many places. This occurs precisely when a place expression is
/// used in a *value context[^1].* For example:
///
/// - `let _ = place;`
/// - `match place { /* ... */ }`
/// - `function(place)`
///
/// When a place expression is being read, its handle must implement this trait.
/// Additionally, the type stored in the place must be [`Copy`], or the handle
/// must also implement [`MovePlace`]. Otherwise a compiler error will be
/// emitted that the value cannot be moved out.
///
/// [`Self::Target`]: PlaceHandle::Target
/// [^1]: <https://doc.rust-lang.org/nightly/reference/expressions.html#r-expr.move>
pub unsafe trait ReadPlace: PlaceHandle {
    /// The access permissions to the place required by [`Self::read_place`].
    const ACCESS: AccessKind;
    /// Whether [`Self::read_place`] is safe when [`Self::ACCESS`] is honored.
    ///
    /// This constant controls whether the compiler will require writing an
    /// `unsafe` block around reading from a place that uses this handle.
    const SAFE: bool;

    /// Read from the place.
    ///
    /// # Safety
    ///
    /// This handle must have [`Self::ACCESS`] permissions for the duration of
    /// this method call.
    unsafe fn read_place(self) -> Self::Target;
}

/// `let _ = place;` -- Move out of a place.
///
/// When reading from a place, the type of the contents of the place must
/// implement [`Copy`], or the handle must implement this trait. This trait
/// allows moving out of the place, leaving it in a partially initialized state.
///
/// When implementing this trait, [`DropPlace`] should almost always also be
/// implemented, since otherwise dropping a partially moved-out proxy is not
/// permitted. Additionally, the proxy should implement [`DropHusk`] for the
/// same reason.
///
/// The actual move-out operation is performed by reading the place
/// ([`ReadPlace::read_place`]) and then changing the borrow checker state of
/// this place to uninitialized.
pub unsafe trait MovePlace: ReadPlace {}

/// `place = value;` -- Write to a place.
///
/// Writing to a place is done by writing the place on the left hand side of an
/// assignment expression.
pub unsafe trait WritePlace: PlaceHandle {
    /// The access permissions to the place required by [`Self::write_place`].
    const ACCESS: AccessKind;
    /// Whether [`Self::write_place`] is safe when [`Self::ACCESS`] is honored.
    ///
    /// This constant controls whether the compiler will require writing an
    /// `unsafe` block around writing to a place that uses this handle.
    const SAFE: bool;

    /// Write to the place.
    ///
    /// # Safety
    ///
    /// This handle must have [`Self::ACCESS`] permissions for the duration of
    /// this method call.
    unsafe fn write_place(self, value: Self::Target);
}

/// `&place`/`@place` -- Borrow a place as `Output`.
///
/// Borrowing a place creates a pointer to the place. This operation is generic
/// over the resulting pointer type and borrowing the same place with many
/// different pointer types is supported.
///
/// There are a few ways to spell borrowing a place:
///
/// - `&place` and `&mut place` --- resulting in `Output = &_` and `Output = &mut
///   _` respectively,
/// - `&raw const place` and `&raw mut place` --- `Output = *const _` and
///   `Output = *mut _`
/// - `@place` and `@<$ty> place` --- `Output = _` and `Output = $ty`
///
/// All of these are desugared to
/// `BorrowPlace::<Output>::borrow(handle!($place))`. Where `handle!` is
/// explained in the [section on place
/// expressions](self#places-and-place-expressions)
pub unsafe trait BorrowPlace<Output>: PlaceHandle {
    /// The access permissions to the place required by [`Self::borrow`].
    const ACCESS: AccessKind;
    /// The timing of the access permissions of [`Self::borrow`].
    type Timing: Timing;
    /// Whether [`Self::borrow`] is safe when [`Self::ACCESS`] is honored for
    /// [`Self::Timing`].
    ///
    /// This constant controls whether the compiler will require writing an
    /// `unsafe` block around borrowing a place that uses this handle.
    const SAFE: bool;

    /// Borrow the place using `Output`.
    ///
    /// # Safety
    ///
    /// This handle must have [`Self::ACCESS`] permissions for the duration of
    /// [`Self::Timing`].
    unsafe fn borrow(self) -> Output;
}

/// `place[idx]` -- Enable indexing into `Self`.
///
/// Indexing is only supported for certain places: namely those handles `H`,
/// for which `Self` implements [`IndexPlace<Idx, H, _, _>`].
///
/// In a way, this trait is a generic version of [`PlaceProxy`], since indexing
/// allows changing the type based on the type of the index.
pub trait Indexable<Idx> {
    /// The type of the place expression `self[idx]`.
    type Element: ?Sized;
}

/// `place[idx]` -- Index into `Self` via the handle `H`.
///
/// The same way that [`Indexable`] is the generic version of [`PlaceProxy`],
/// this trait is the generic version of [`DerefPlace`].
///
/// Indexing -- like dereferencing -- also performs two accesses from the
/// borrow-checker's perspective:
///
/// - the place that comes out of the index operation (`place[idx]`), and
/// - the place that is being indexed into (`place`).
///
/// For this reason, there are two [`AccessKind`] constants and two [`Timing`]
/// generics, see the documentation of [`DerefPlace`] for more information about
/// them.
pub unsafe trait IndexPlace<Idx, H, PointeeTiming, PointerTiming>:
    Indexable<Idx>
where
    H: PlaceHandle<Target = Self>,
    PointeeTiming: Timing,
    PointerTiming: Timing,
{
    /// The type of handles to indexed elements.
    type ElementHandle: PlaceHandle<Target = Self::Element>;

    /// The access permissions to the contents of the place handled by `Self`
    /// required by [`Self::index`].
    const POINTEE_ACCESS: AccessKind;
    /// The access permissions to the place handled by `Self` required by
    /// [`Self::index`].
    const POINTER_ACCESS: AccessKind;
    /// Whether [`Self::index`] is safe when [`Self::POINTEE_ACCESS`] and
    /// [`Self::POINTER_ACCESS`] are honored for `PointeeTiming` and
    /// `PointerTiming` respectively.
    ///
    /// This constant controls whether the compiler will require writing an
    /// `unsafe` block around indexing into a place that uses the `H` handle.
    const SAFE: bool;

    /// Indexes into the value stored at `H`.
    fn index(handle: H, idx: Idx) -> Self::ElementHandle;
}

/// `*place` -- Dereference the contents of a place.
///
/// Dereferencing from a borrow-checker perspective requires access to two
/// places:
///
/// - the place that comes out of the dereference operation (`*place`), and
/// - the place that's being dereferenced (`place`).
///
/// For this reason, the dereference operation has two associated constants of
/// type [`AccessKind`] that specify the access permissions required for the two
/// accesses and two [`Timing`] generics[^1] called `PointeeTiming` and
/// `PointerTiming`.
///
/// Given a pointer `ptr`, the place that comes out of the dereference is `*ptr`
/// and the place that is being dereferenced is `ptr`. The `PointerTiming`
/// generic and [`Self::POINTER_ACCESS`] constant control how the borrow checker
/// treats the access to `ptr`, while `PointeeTiming` and
/// [`Self::POINTEE_ACCESS`] specify the kind of access to `*ptr`.
///
/// Utilizing this fact, one can encode that a pointer can be invalidated
/// without invalidating pointers that were derived from dereferenced pointers.
/// This is the case for mutable references:
///
/// ```ignore
/// fn overwrite_nested<'a>(ptr: &mut &'a mut Struct, make: impl FnOnce() -> &'a mut Struct) {
///     let a: &'a mut Field = &mut (**ptr).field;
///     *ptr = make();
///     let b: &'a mut Field = &mut (**ptr).field;
///
///     mem::swap(a, b); // can use both `a` and `b`!
/// }
/// ```
///
/// In this case, dereferencing `&mut` has [`Instant`](borrowck::Instant) as the
/// `PointerTiming`, which results in never invalidating derived pointers when
/// the original is used for something else.
///
/// [^1]: They can't be associated types, because the timing [`Lifetime<'a>`]
///       has a lifetime. If they were associated types, one could only use
///       [`Lifetime<'a>`] if the handle also had that same lifetime. Because
///       handles generally want to allow shortening lifetimes, the timing needs
///       to introduce a fresh lifetime.
///
/// [`Lifetime<'a>`]: borrowck::Lifetime
pub unsafe trait DerefPlace<PointeeTiming, PointerTiming>:
    PlaceHandle
where
    Self::Target: CreateHandle<PointeeTiming>,
    PointeeTiming: Timing,
    PointerTiming: Timing,
{
    /// The access permissions to the contents of the place handled by `Self`
    /// required by [`Self::deref_place`].
    const POINTEE_ACCESS: AccessKind;
    /// The access permissions to the place handled by `Self` required by
    /// [`Self::deref_place`].
    const POINTER_ACCESS: AccessKind;
    /// Whether [`Self::deref_place`] is safe when [`Self::POINTEE_ACCESS`] and
    /// [`Self::POINTER_ACCESS`] are honored for `PointeeTiming` and
    /// `PointerTiming` respectively.
    ///
    /// This constant controls whether the compiler will require writing an
    /// `unsafe` block around dereferencing a place that uses the this handle.
    const SAFE: bool;

    unsafe fn deref_place(
        self,
    ) -> <Self::Target as CreateHandle<PointeeTiming>>::Handle;
}

/// `place.field` -- Project a handle to a subplace.
pub unsafe trait ProjectPlace<S>: PlaceHandle
where
    S: Subplace<Source = Self::Target>,
{
    type Projected: PlaceHandle<Target = S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected;
}

/// Wrap a place, exposing some of its subplaces.
pub trait PlaceWrapper {
    type Inner: ?Sized;
}

/// `place.field` -- Expose a modified subplace of the wrapped place.
pub unsafe trait WrapPlace<S>: PlaceWrapper
where
    S: Subplace<Source = Self::Inner>,
{
    type Wrapped: Subplace<Source = Self>;

    fn wrap(subplace: S) -> Self::Wrapped;
}

/// Drop the contents of a place.
///
/// This operation should only drop the value at the place and not invalidate
/// the proxy itself. A new value might be moved back in later with
/// [`WritePlace`].
///
/// Calls to [`Self::drop_place`] are emitted by the compiler as part of
/// dropping a [`PlaceProxy`] that's partially moved out.
pub unsafe trait DropPlace: PlaceHandle {
    unsafe fn drop_place(self);
}

/// Destroy a [`PlaceProxy`] where its contents have been moved out/dropped.
///
/// This is essentially like [`Drop`], but supports the situation where the
/// value stored in the place represented by `Self` have been moved out,
/// dropped, or never initialized to begin with.
///
/// The borrow checker tracks the initialization state of each (sub)place. When
/// a [`PlaceProxy`] supports moving values out (i.e. when its handle implements
/// [`MovePlace`]), then after moving out the entire value, the allocation might
/// still be live. Since the value is no longer populated, calling
/// [`Drop::drop`] as normal would result in using a moved-out value, which can
/// result in a double-free.
///
/// Instead, this trait is combined with [`DropPlace`], which the compiler uses
/// to drop any not-moved-out subplaces and then drops the allocation (or any
/// other data that the proxy had) through this trait.
pub unsafe trait DropHusk: PlaceProxy {
    /// Destroy the proxy associated with the place of `this`.
    unsafe fn drop_husk(this: *mut Self);
}

/// Obtain the metadata of the contents of a place.
pub unsafe trait ReadMetadata: PlaceHandle {
    fn metadata(self) -> Metadata<Self::Target>;
}

/// Obtain the discriminant of the contents of a place.
pub unsafe trait ReadVariant: PlaceHandle
where
    Self::Target: Matchable,
{
    unsafe fn read_variant(self) -> &'static str;
}

/// Cast a handle to a place to a subtype.
pub unsafe trait VariantPlace<const VARIANT: &'static str>:
    ReadVariant
where
    Self::Target: Matchable,
    Self::Target: HasVariant<VARIANT>,
{
    type ToVariant: PlaceHandle<Target = VariantType<Self::Target, VARIANT>>;

    unsafe fn cast(self) -> Self::ToVariant;
}
