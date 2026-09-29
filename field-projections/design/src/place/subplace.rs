#![deny(unused_imports, reason = "to error with `ensure_full_reexport!`")]
#![deny(
    unfulfilled_lint_expectations,
    reason = "to error with `ensure_full_reexport!`"
)]

use macros::ensure_full_reexport;

ensure_full_reexport!(
    mod indexes;
    mod traits;
    mod transmuted;
);

pub use self::{
    indexes::{
        ArrayIndex,
        SliceIndex,
    },
    traits::{
        Field,
        HasVariant,
        Matchable,
        Subplace,
        VariantType,
    },
    transmuted::{
        TransmutedField,
        TransmutedSubplace,
    },
};
