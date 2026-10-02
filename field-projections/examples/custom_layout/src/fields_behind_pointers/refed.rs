use std::mem;

use design::{
    ops::place::{
        CreateHandle,
        PlaceHandle,
        PlaceProxy,
        ProjectPlace,
        borrowck::{
            AccessKind,
            Instant,
        },
    },
    place::{
        MutHandle,
        RefHandle,
    },
};

use crate::{
    ___lang_limits,
    Struct,
    field_of,
};

pub struct RefedFieldStruct<'a> {
    x: i32,
    y: &'a mut [i32; 1024],
}

impl PlaceProxy for RefedFieldStruct<'_> {
    type Target = Struct;
}

unsafe impl<'a> CreateHandle<Instant> for RefedFieldStruct<'a> {
    type Handle = RefedFieldStructHandle<'a>;
    const ACCESS: AccessKind = AccessKind::Exclusive;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        RefedFieldStructHandle(this)
    }
}

pub struct RefedFieldStructHandle<'a>(*const RefedFieldStruct<'a>);

impl PlaceHandle for RefedFieldStructHandle<'_> {
    type Target = Struct;
}

unsafe impl ProjectPlace<field_of!(Struct, x)> for RefedFieldStructHandle<'_> {
    type Projected = RefHandle<'static, i32>;

    unsafe fn project_place(self, _: field_of!(Struct, x)) -> Self::Projected {
        let ptr: *const i32 = unsafe { &raw const (*self.0).x };
        unsafe { mem::transmute(ptr) }
    }
}

unsafe impl<'a> ProjectPlace<field_of!(Struct, y)>
    for RefedFieldStructHandle<'a>
{
    type Projected = MutHandle<'a, [i32; 1024]>;

    unsafe fn project_place(self, _: field_of!(Struct, y)) -> Self::Projected {
        let ptr: *const &'a mut [i32; 1024] = unsafe { &raw const (*self.0).y };
        unsafe { <&'a mut [_; _] as CreateHandle<_>>::handle_from_raw(ptr) }
    }
}
