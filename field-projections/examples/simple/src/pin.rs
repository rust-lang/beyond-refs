/// How [`Pin`] acts as a pointer wrapper with pin info on fields.
///
/// For this example, we're assuming that the pinning information is stored on
/// the field via the [`PinnableSubplace`] trait. To highlight the two different
/// behaviors, we choose to make `a` not structurally pinned and `b`
/// structurally pinned.
///
/// ## Sugared Code
///
/// ```
/// let mut x = pin!(Struct { a: 42, b: 24 });
/// let a: &mut u32 = @x.a;
/// let b: Pin<&mut i32> = @x.b;
/// *a += *b as u32;
/// ```
///
/// ## Desugared Code
///
/// ```
/// ==== DESUGARED ====
/// ```
#[cfg_attr(doc, design::utils::desugared)]
pub fn pin_field_tracked() {
    let x = pin!(Struct { a: 42, b: 24 });
    let a: &mut u32 = unsafe {
        let hdl: LocalHandle<Pin<&mut Struct>> = LocalHandle::new(&raw const x);
        let hdl: PinnedHandle<MutHandle<'_, Struct>> =
            DerefPlace::deref_place(hdl);
        let subplace = <field_of!(Struct, a)>::default();
        let hdl: MutHandle<'_, u32> =
            ProjectPlace::project_place(hdl, subplace);
        BorrowPlace::<&mut u32>::borrow(hdl)
    };
    let b: Pin<&mut i32> = unsafe {
        let hdl: LocalHandle<Pin<&mut Struct>> = LocalHandle::new(&raw const x);
        let hdl: PinnedHandle<MutHandle<'_, Struct>> =
            DerefPlace::deref_place(hdl);
        let subplace = <field_of!(Struct, b)>::default();
        let hdl: PinnedHandle<MutHandle<'_, i32>> =
            ProjectPlace::project_place(hdl, subplace);
        BorrowPlace::<Pin<&mut i32>>::borrow(hdl)
    };
    AddAssign::add_assign(
        unsafe {
            let hdl: LocalHandle<&mut u32> = LocalHandle::new(&raw const a);
            let hdl: MutHandle<'_, u32> = DerefPlace::deref_place(hdl);
            BorrowPlace::<&mut u32>::borrow(hdl)
        },
        unsafe {
            let hdl: LocalHandle<Pin<&mut i32>> =
                LocalHandle::new(&raw const b);
            let hdl: PinnedHandle<MutHandle<'_, i32>> =
                DerefPlace::deref_place(hdl);
            ReadPlace::read_place(hdl)
        } as u32,
    );
}
