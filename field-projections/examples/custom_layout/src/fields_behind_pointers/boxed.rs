use std::mem;

use design::{
    boxed::BoxHandle,
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
    place::MutHandle,
};

use crate::{
    ___lang_limits,
    Struct,
    field_of,
};

pub struct BoxedFieldStruct {
    x: i32,
    y: Box<[i32; 1024]>,
}

impl PlaceProxy for BoxedFieldStruct {
    type Target = Struct;
}

unsafe impl CreateHandle<Instant> for BoxedFieldStruct {
    type Handle = BoxedFieldStructHandle;
    const ACCESS: AccessKind = AccessKind::Exclusive;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        BoxedFieldStructHandle(this)
    }
}

pub struct BoxedFieldStructHandle(*const BoxedFieldStruct);

impl PlaceHandle for BoxedFieldStructHandle {
    type Target = Struct;
}

unsafe impl ProjectPlace<field_of!(Struct, x)> for BoxedFieldStructHandle {
    type Projected = MutHandle<'static, i32>;

    unsafe fn project_place(self, _: field_of!(Struct, x)) -> Self::Projected {
        let ptr: *const i32 = unsafe { &raw const (*self.0).x };
        unsafe { mem::transmute(ptr) }
    }
}

unsafe impl ProjectPlace<field_of!(Struct, y)> for BoxedFieldStructHandle {
    type Projected = BoxHandle<[i32; 1024]>;

    unsafe fn project_place(self, _: field_of!(Struct, y)) -> Self::Projected {
        let ptr: *const Box<[i32; 1024]> = unsafe { &raw const (*self.0).y };
        unsafe { <Box<_> as CreateHandle<_>>::handle_from_raw(ptr) }
    }
}
