#![deny(unused_imports, reason = "to error with `ensure_full_reexport!`")]
#![deny(
    unfulfilled_lint_expectations,
    reason = "to error with `ensure_full_reexport!`"
)]

use macros::ensure_full_reexport;

ensure_full_reexport!(
    mod handles;
    mod subplace;
);

pub use self::{
    handles::{
        FallibleHandle,
        LocalHandle,
        MutHandle,
        RefHandle,
    },
    subplace::{
        ArrayIndex,
        Field,
        HasVariant,
        Matchable,
        SliceIndex,
        Subplace,
        TransmutedField,
        TransmutedSubplace,
        VariantType,
    },
};
