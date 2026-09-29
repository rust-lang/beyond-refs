//! # *Handles* -- a Field Projection Design
//!
//! This crate is not only meant as a wiki and up-to-date version of the design,
//! but also as a testing ground for new ideas. It also is very much intended to
//! be used by people who would like to figure out if field projections can help
//! in their use-case.
//!
//! <p class="warning"> This crate essentially enables writing
//! (<code>unsafe</code>) desugared code and running it like any other Rust
//! code. It is <strong>not</strong> designed for production, only for
//! prototypes. It makes no guarantees on soundness, API stability, or
//! correctness. </p>
//!
//! Another word of warning: writing code using this crate means manually
//! desugaring the code that you actually wish to write; it is tedious and
//! error-prone. But depending on your use-case, you might already be writing
//! similarly convoluted code.
//!
//! ## Design Overview
//!
//! This design builds on top of the place centered field projection design
//! invented by Nadrieril. The core idea that this design adds is that places
//! are represented using *handles*. Every place expression has a corresponding
//! handle.
//!
//! You can think of a handle as a pointer to the place it represents--- most of
//! the time, that's also how it's implemented. But there also are virtual
//! places where the handle can be a ZST. Furthermore, handles can of course
//! contain more information than a "normal pointer", so access permissions,
//! multiple pointers etc.
//!
//! ## Crate Overview
//!
//! This crate has the same structure as the standard library, as many parts are
//! intended to be eventually incorporated into it. In addition, this crate also
//! contains compatibility APIs and workarounds for language limitations that
//! need compiler support.
//!
//!
//! Here is a list of contents sorted by relevance (the most important item is
//! the first):
//! 1. [`ops::place`] -- all place operations that can be implemented on
//!    handles.
//! 2. [`place`] -- several non-operation place traits, utility types, and
//!    handles for builtin types and local variables.
//! 3. [`lang_limits`] -- APIs that work around compiler & language limitations,
//!    required to write examples.
//! 4. [`cell`], [`mem`], [`pin`], [`ptr`], [`sync`], [`mod@vec`] --
//!    extensions to standard library modules that integrate them into the place
//!    operations.
//!
//! ## Examples
//!
//! One of the main reasons for this crate is to showcase how code would be
//! desugared with the current approach. In the list of crates on the left side,
//! you can find several examples. They show both the sugared and desugared
//! versions of the code in the documentation and are run as tests in this
//! repository.
//!
//! A good starting point is the `E_simple` crate, which gives a gentle
//! introduction into how the existing reference types would work in a place
//! world.

#![expect(incomplete_features)]
#![allow(unused_features)]
// Needed for proper support of `?Sized` types.
#![feature(ptr_metadata)]
// Needed for enum support (`&'static str` in const generics).
#![feature(adt_const_params)]
#![feature(unsized_const_params)]
#![feature(sync_unsafe_cell)]
#![feature(vec_as_non_null)]
#![feature(proc_macro_hygiene)]
#![feature(auto_traits)]
#![feature(phantom_variance_markers)]

pub mod boxed;
pub mod cell;
pub mod lang_limits;
pub mod mem;
pub mod ops;
pub mod pin;
pub mod place;
pub mod ptr;
pub mod sync;
pub mod utils;
pub mod vec;
