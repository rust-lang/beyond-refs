#![expect(incomplete_features, dead_code)]
#![feature(auto_traits, negative_impls, offset_of_enum)]
#![feature(adt_const_params)]
#![feature(unsized_const_params)]

use design::{
    lang_limits::adt_reflect,
    ops::place::{
        ProjectPlace,
        ReadPlace,
        ReadVariant,
        VariantPlace,
        WrapPlace,
    },
    place::LocalHandle,
};

use crate::unnormalized::{
    GenericArgs,
    Unnormalized,
    UnwrapUnnormalize,
};

pub mod unnormalized;

adt_reflect!(
    pub enum TyKind {
        Int { int: IntTy },
        FnPtr { sig: Sig },
    }
);

#[derive(Debug)]
pub struct IntTy;
#[derive(Debug)]
pub struct Sig(GenericArgs);

/// Workarounds for language/compiler limitations.
///
/// These would go away when more features are added to the compiler/language:
/// - negative reasoning for overlap checks of impls
mod lang_limits {
    use super::*;

    impl UnwrapUnnormalize for variant_of!(TyKind, Int) {
        type Unwrapped = Self;

        fn from_wrapped(this: Self) -> Self::Unwrapped {
            this
        }
    }

    impl UnwrapUnnormalize for variant_of!(TyKind, FnPtr) {
        type Unwrapped = Unnormalized<Self>;

        fn from_wrapped(this: Self) -> Self::Unwrapped {
            this.into()
        }
    }

    impl UnwrapUnnormalize for Sig {
        type Unwrapped = Unnormalized<Self>;

        fn from_wrapped(this: Self) -> Self::Unwrapped {
            this.into()
        }
    }
}

/// ```
/// let ty_kind: Unnormalized<TyKind> = ...;
/// match ty_kind {
///     TyKind::Int(int_ty) => {
///         let x: IntTy = int_ty;
///         println!("{x:?}");
///     }
///     TyKind::FnPtr(sig) => {
///         let x: Unnormalized<Sig> = sig;
///         println!("{x:?}");
///     }
/// }
/// ```
fn demo(ty_kind: Unnormalized<TyKind>) {
    let hdl: LocalHandle<Unnormalized<TyKind>> =
        unsafe { LocalHandle::new(&raw const ty_kind) };
    let discr: &'static str = unsafe { ReadVariant::read_variant(hdl) };
    match discr {
        "Int" => {
            let int_ty = unsafe {
                //    variant_of!(Enum, Variant)
                // == pattern_type!(Enum is Variant(..))
                let variant_hdl: LocalHandle<variant_of!(TyKind, Int)> =
                    VariantPlace::<"Int">::cast(hdl);

                let int_ty_subplace = <field_of!(TyKind::Int, int)>::default();

                let int_ty_hdl: LocalHandle<IntTy> =
                    ProjectPlace::project_place(variant_hdl, int_ty_subplace);

                ReadPlace::read_place(int_ty_hdl)
            };
            let x: IntTy = int_ty;
            println!("{x:?}");
        }
        "FnPtr" => {
            let sig = unsafe {
                let variant_hdl: LocalHandle<
                    Unnormalized<variant_of!(TyKind, FnPtr)>,
                > = VariantPlace::<"FnPtr">::cast(hdl);

                let sig_subplace = <field_of!(TyKind::FnPtr, sig)>::default();
                let sig_subplace_wrapped = Unnormalized::wrap(sig_subplace);

                let hdl: LocalHandle<Unnormalized<Sig>> =
                    ProjectPlace::project_place(
                        variant_hdl, sig_subplace_wrapped,
                    );

                ReadPlace::read_place(hdl)
            };
            let x: Unnormalized<Sig> = sig;
            println!("{x:?}");
        }
        _ => unreachable!(),
    }
}

#[cfg_attr(test, test)]
fn main() {
    let ty_kind = TyKind::Int { int: IntTy };
    demo(ty_kind.into());
    let ty_kind = TyKind::FnPtr {
        sig: Sig(GenericArgs::default()),
    };
    demo(ty_kind.into());
}
