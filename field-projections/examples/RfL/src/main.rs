#![feature(unsafe_pinned)]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(arbitrary_self_types)]

pub mod examples;
pub mod mutex;
pub mod opaque;
pub mod overwrite;
pub mod rcu;
pub mod untrusted;

mod bindings;

fn main() {}
