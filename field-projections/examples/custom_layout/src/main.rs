#![warn(macro_expanded_macro_exports_accessed_by_absolute_paths)]
use design::lang_limits::adt_reflect;

pub mod fields_behind_pointers;

adt_reflect!(
    pub struct Struct {
        pub x: i32,
        pub y: [i32; 1024],
    }
);

fn main() {}
