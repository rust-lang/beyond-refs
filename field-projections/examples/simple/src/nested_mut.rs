#[test]
#[doc(hidden)]
fn nested_mut_sugared() {
    let mut x = Struct { a: 42, b: 24 };
    let mut xx = Struct { a: 0, b: -42 };

    let mut y = &mut x;
    let z = &mut y;

    let a = &mut (**z).a;
    let b = &mut z.b;

    *z = &mut xx;

    *a += 1;
    *b += 1;

    let aa = &mut z.a;
    let bb = &mut z.b;

    *aa += 1;
    *bb += 1;

    *a += 1;
    *b += 1;

    assert_eq!(*a, 42 + 2);
    assert_eq!(*b, 24 + 2);
    assert_eq!(*aa, 0 + 1);
    assert_eq!(*bb, -42 + 1);
}

/// Nested mutable references work the same way as today.
///
/// ## Sugared Code
///
/// ```
/// # struct Struct { a: u32, b: i32 }
/// let mut x = Struct { a: 42, b: 24 };
/// let mut xx = Struct { a: 0, b: -42 };
///
/// let mut y = &mut x;
/// let z = &mut y;
///
/// let a = &mut (**z).a;
/// let b = &mut z.b;
///
/// *z = &mut xx;
///
/// *a += 1;
/// *b += 1;
///
/// let aa = &mut z.a;
/// let bb = &mut z.b;
///
/// *aa += 1;
/// *bb += 1;
///
/// *a += 1;
/// *b += 1;
///
/// print(*a);
/// print(*aa);
/// print(*b);
/// print(*bb);
/// ```
///
/// Note that the mutable reference that we're accessing sits behind another
/// mutable reference, this makes it more complex for the borrow checker
/// tracking. We overwrite the inner mutable reference and then take new borrows
/// to it without invalidating the previous borrows at all. This requires
/// special support from the borrow checker that knows that overwriting a nested
/// mutable reference doesn't invalidate any borrows to the inner one (as it
/// continues to exist).
///
/// ## Desugared code
///
/// ```
/// ==== DESUGARED ====
/// ```
///
/// [`DerefMut`]: core::ops::DerefMut
/// [`Subplace`]: design::place::Subplace
#[cfg_attr(test, test)]
#[cfg_attr(doc, design::utils::desugared)]
pub fn nested_mut() {
    let mut x = Struct { a: 42, b: 24 };
    let mut xx = Struct { a: 0, b: -42 };

    let mut y = unsafe {
        let hdl: LocalHandle<Struct> = LocalHandle::new(&raw mut x);
        BorrowPlace::<&mut Struct>::borrow(hdl)
    };
    let z = unsafe {
        let hdl: LocalHandle<&mut Struct> = LocalHandle::new(&raw mut y);
        BorrowPlace::<&mut &mut Struct>::borrow(hdl)
    };

    let a: &mut u32 = unsafe {
        let hdl: LocalHandle<&mut &mut Struct> = LocalHandle::new(&raw const z);
        let hdl: MutHandle<'_, &mut Struct> = DerefPlace::deref_place(hdl);
        let hdl: MutHandle<'_, Struct> = DerefPlace::deref_place(hdl);
        let a_subplace = <field_of!(Struct, a)>::default();
        let hdl: MutHandle<'_, u32> =
            ProjectPlace::project_place(hdl, a_subplace);
        BorrowPlace::<&mut u32>::borrow(hdl)
    };
    let b: &mut i32 = unsafe {
        let hdl: LocalHandle<&mut &mut Struct> = LocalHandle::new(&raw const z);
        let hdl: MutHandle<'_, &mut Struct> = DerefPlace::deref_place(hdl);
        let hdl: MutHandle<'_, Struct> = DerefPlace::deref_place(hdl);
        let b_subplace = <field_of!(Struct, b)>::default();
        let hdl: MutHandle<'_, i32> =
            ProjectPlace::project_place(hdl, b_subplace);
        BorrowPlace::<&mut i32>::borrow(hdl)
    };

    {
        let tmp: &mut Struct = unsafe {
            let hdl: LocalHandle<Struct> = LocalHandle::new(&raw mut xx);
            BorrowPlace::<&mut Struct>::borrow(hdl)
        };
        unsafe {
            let hdl: LocalHandle<&mut &mut Struct> =
                LocalHandle::new(&raw const z);
            let hdl: MutHandle<'_, &mut Struct> = DerefPlace::deref_place(hdl);
            DropPlace::drop_place(hdl);
            WritePlace::write_place(hdl, tmp);
        }
    }

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

    let aa: &mut u32 = unsafe {
        let hdl: LocalHandle<&mut &mut Struct> = LocalHandle::new(&raw const z);
        let hdl: MutHandle<'_, &mut Struct> = DerefPlace::deref_place(hdl);
        let hdl: MutHandle<'_, Struct> = DerefPlace::deref_place(hdl);
        let a_subplace = <field_of!(Struct, a)>::default();
        let hdl: MutHandle<'_, u32> =
            ProjectPlace::project_place(hdl, a_subplace);
        BorrowPlace::<&mut u32>::borrow(hdl)
    };
    let bb: &mut i32 = unsafe {
        let hdl: LocalHandle<&mut &mut Struct> = LocalHandle::new(&raw const z);
        let hdl: MutHandle<'_, &mut Struct> = DerefPlace::deref_place(hdl);
        let hdl: MutHandle<'_, Struct> = DerefPlace::deref_place(hdl);
        let b_subplace = <field_of!(Struct, b)>::default();
        let hdl: MutHandle<'_, i32> =
            ProjectPlace::project_place(hdl, b_subplace);
        BorrowPlace::<&mut i32>::borrow(hdl)
    };

    AddAssign::add_assign(
        unsafe {
            let hdl: LocalHandle<&mut u32> = LocalHandle::new(&raw const aa);
            let hdl: MutHandle<'_, u32> = DerefPlace::deref_place(hdl);
            BorrowPlace::<&mut u32>::borrow(hdl)
        },
        1,
    );
    AddAssign::add_assign(
        unsafe {
            let hdl: LocalHandle<&mut i32> = LocalHandle::new(&raw const bb);
            let hdl: MutHandle<'_, i32> = DerefPlace::deref_place(hdl);
            BorrowPlace::<&mut i32>::borrow(hdl)
        },
        1,
    );

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

    print(unsafe {
        let hdl: LocalHandle<&mut u32> = LocalHandle::new(&raw const a);
        let hdl: MutHandle<'_, u32> = DerefPlace::deref_place(hdl);
        ReadPlace::read_place(hdl)
    });
    print(unsafe {
        let hdl: LocalHandle<&mut u32> = LocalHandle::new(&raw const aa);
        let hdl: MutHandle<'_, u32> = DerefPlace::deref_place(hdl);
        ReadPlace::read_place(hdl)
    });
    print(unsafe {
        let hdl: LocalHandle<&mut i32> = LocalHandle::new(&raw const b);
        let hdl: MutHandle<'_, i32> = DerefPlace::deref_place(hdl);
        ReadPlace::read_place(hdl)
    });
    print(unsafe {
        let hdl: LocalHandle<&mut i32> = LocalHandle::new(&raw const bb);
        let hdl: MutHandle<'_, i32> = DerefPlace::deref_place(hdl);
        ReadPlace::read_place(hdl)
    });
}
