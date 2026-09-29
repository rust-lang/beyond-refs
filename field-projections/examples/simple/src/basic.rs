#[test]
#[doc(hidden)]
fn basic_sugared() {
    let mut x = Struct { a: 42, b: 24 };
    let y = &mut x;
    let a = &mut (*y).a;
    let b = &mut y.b;
    *a += 1;
    *b += 1;
    assert_eq!(*a, 42 + 1);
    assert_eq!(*b, 24 + 1);
}

/// A small example showing the power of the place based approach over
/// [`DerefMut`].
///
/// ## Sugared Code
///
/// ```
/// # struct Struct { a: u32, b: i32 }
/// let mut x = Struct { a: 42, b: 24 };
/// let y = &mut x;
/// let a = &mut (*y).a;
/// let b = &mut y.b;
/// *a += 1;
/// *b += 1;
/// ```
///
/// There are several things happening here that are notable:
/// - in the assignment of `b`, there is an implicit deref of `y`.
/// - we're able to use both `a` *and* `b`; if `&mut` was implemented with
///   [`DerefMut`] instead, there would be two calls to `deref_mut`. The second
///   one (the one that creates `b`) would invalidate `a`, since it would borrow
///   the entirety of `y`. The place-based approach allows us to make use of the
///   borrow checker that knows that `.a` and `.b` are disjoint, thus allowing
///   the simultaneous mutable borrow.
///
/// ## Desugared code
///
/// The desugaring is rather straight forward, but can be very intimidating for
/// the first time. Here's a quick rundown what the desugaring process looks like:
///
/// 1. Place expressions are made explicit. In this case, the subexpression
///    `y.b` in the `let b` statement is expanded to `(*y).b`.
/// 2. Temporary variables are made explicit (in this example, there are no
///    temporaries).
/// 3. Place expressions are recursively turned into place handles:
///    - Local variable accesses of `$var` are turned into
///      <code>[LocalHandle]::new(&raw {const,mut} $var)</code> depending on
///      their mutability.
///    - Dereference operations of `$hdl` are turned into
///      <code>[DerefPlace]::deref_place($hdl)</code>
///    - Field access operations of the handle `$hdl` with the field `$field`
///      are turned into <code>[ProjectPlace]::project_place($hdl, $subplace)</code>
///      where `$subplace` is the [`Subplace`]-representing value created based
///      on the type that `$hdl` points at and the field `$field`.
///    - ... (there are more desugarings, but they don't occur in this example)
/// 4. Place operations on place expressions are replaced by the respective
///    operation on the computed handle:
///    - [`BorrowPlace`]
///    - [`ReadPlace`] (not shown here)
///    - [`WritePlace`] (not shown here)
///    - ... (again there are more advanced operations)
///
/// Note that the borrow checker is intended to run before this desugaring,
/// which is why the desugared `unsafe` is sound.
///
/// ```
/// ==== DESUGARED ====
/// ```
///
/// [`DerefMut`]: core::ops::DerefMut
/// [`Subplace`]: design::place::Subplace
#[cfg_attr(doc, design::utils::desugared)]
pub fn basic() {
    let mut x = Struct { a: 42, b: 24 };
    let y = unsafe {
        let hdl: LocalHandle<Struct> = LocalHandle::new(&raw mut x);
        BorrowPlace::<&mut Struct>::borrow(hdl)
    };
    let a: &mut u32 = unsafe {
        let hdl: LocalHandle<&mut Struct> = LocalHandle::new(&raw const y);
        let hdl: MutHandle<'_, Struct> = DerefPlace::deref_place(hdl);
        let a_subplace = <field_of!(Struct, a)>::default();
        let hdl: MutHandle<'_, u32> =
            ProjectPlace::project_place(hdl, a_subplace);
        BorrowPlace::<&mut u32>::borrow(hdl)
    };
    let b: &mut i32 = unsafe {
        let hdl: LocalHandle<&mut Struct> = LocalHandle::new(&raw const y);
        let hdl: MutHandle<'_, Struct> = DerefPlace::deref_place(hdl);
        let b_subplace = <field_of!(Struct, b)>::default();
        let hdl: MutHandle<'_, i32> =
            ProjectPlace::project_place(hdl, b_subplace);
        BorrowPlace::<&mut i32>::borrow(hdl)
    };
    AddAssign::add_assign(
        unsafe {
            let hdl: LocalHandle<&mut u32> = LocalHandle::new(&raw const a);
            let hdl: MutHandle<'_, u32> = DerefPlace::deref_place(hdl);
            BorrowPlace::<&mut u32>::borrow(hdl)
        },
        1,
    );
    AddAssign::add_assign(
        unsafe {
            let hdl: LocalHandle<&mut i32> = LocalHandle::new(&raw const b);
            let hdl: MutHandle<'_, i32> = DerefPlace::deref_place(hdl);
            BorrowPlace::<&mut i32>::borrow(hdl)
        },
        1,
    );
}
