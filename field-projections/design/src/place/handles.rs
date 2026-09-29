#![deny(unused_imports, reason = "to error with `ensure_full_reexport!`")]
#![deny(
    unfulfilled_lint_expectations,
    reason = "to error with `ensure_full_reexport!`"
)]

use macros::ensure_full_reexport;

ensure_full_reexport!(
    mod fallible;
    mod local;
    mod ref_const;
    mod ref_mut;
);

pub use self::{
    fallible::FallibleHandle,
    local::LocalHandle,
    ref_const::RefHandle,
    ref_mut::MutHandle,
};
