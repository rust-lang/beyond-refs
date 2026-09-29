//! Basic place operation examples.
//!
//! This example crate contains some simple examples along with explanations to
//! help the reader get acquainted with how place operations are desugared.
//! The examples contain both the sugared and desugared versions of the code.
//! The desugared ones also contain all type ascriptions to make reading the
//! code easier.
//!
//! The first example function that you should take a look at is the [`basic`]
//! function, as it explains the desugaring that happens. It also is the
//! simplest of the examples that is available.
//!
//! After that, [`nested_mut`] showcases an interesting and -- while intuitive
//! -- surprising property of [`&mut T`](primitive@reference).
//!
//! Lastly, we showcase how pinning a mutable reference looks like in [`pin`].

use std::{
    fmt::Display,
    mem::forget,
    ops::AddAssign,
    pin::{
        Pin,
        pin,
    },
};

use design::{
    lang_limits::adt_reflect,
    ops::place::{
        BorrowPlace,
        DerefPlace,
        DropHusk,
        DropPlace,
        PlaceHandle,
        ProjectPlace,
        ReadPlace,
        WritePlace,
    },
    pin::{
        PinnableSubplace,
        PinnedHandle,
    },
    place::{
        LocalHandle,
        MutHandle,
    },
};

adt_reflect!(
    pub struct Struct {
        a: u32,
        b: i32,
    }

    pub struct BranchConfig {
        main: String,
        dev: String,
    }
);

unsafe impl PinnableSubplace for field_of!(Struct, a) {
    type StructualPinning<H: PlaceHandle<Target = Self::Target>> = H;

    unsafe fn from_pinned<H: PlaceHandle<Target = Self::Target>>(
        handle: H,
    ) -> Self::StructualPinning<H> {
        handle
    }
}

unsafe impl PinnableSubplace for field_of!(Struct, b) {
    type StructualPinning<H: PlaceHandle<Target = Self::Target>> =
        PinnedHandle<H>;

    unsafe fn from_pinned<H: PlaceHandle<Target = Self::Target>>(
        handle: H,
    ) -> Self::StructualPinning<H> {
        unsafe { PinnedHandle::new_unchecked(handle) }
    }
}

/// Prints a value.
fn print(value: impl Display) {
    println!("{value}");
}

include!("basic.rs");
include!("nested_mut.rs");
include!("pin.rs");
include!("move_out.rs");

#[doc(hidden)]
fn main() {
    basic();
    nested_mut();
    pin_field_tracked();
}
