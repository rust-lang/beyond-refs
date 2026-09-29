use design::{
    ops::place::{
        PlaceWrapper,
        WrapPlace,
    },
    place::{
        HasVariant,
        Matchable,
        Subplace,
        TransmutedSubplace,
        VariantType,
    },
};

#[derive(Debug)]
#[repr(transparent)]
pub struct Unnormalized<T>(T);

impl<T> From<T> for Unnormalized<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

pub trait UnwrapUnnormalize: Sized {
    /// Must be one of:
    /// - `Self`
    /// - `Unnormalized<Self>`
    ///
    /// depending on whether `Self` must be normalized before use.
    type Unwrapped;

    fn from_wrapped(this: Self) -> Self::Unwrapped;
}

pub unsafe auto trait HasNoGenerics {}

impl<T> UnwrapUnnormalize for T
where
    T: HasNoGenerics,
{
    type Unwrapped = T;

    fn from_wrapped(this: Self) -> Self::Unwrapped {
        this
    }
}

#[derive(Debug, Default)]
#[expect(dead_code)]
pub struct GenericArgs(usize);

impl !HasNoGenerics for GenericArgs {}

impl UnwrapUnnormalize for GenericArgs {
    type Unwrapped = Unnormalized<Self>;

    fn from_wrapped(this: Self) -> Self::Unwrapped {
        this.into()
    }
}

/*
 * Would like to write this impl, but can't because negative reasoning doesn't
 * exist.
 *
impl<T: !HasNoGenerics> UnwrapUnnormalize for T {
    type Unwrapped = Unnormalized<T>;

    fn from_wrapped(this: Self) -> Self::Unwrapped {
        Unnormalized(this)
    }
}
*/

impl<T> PlaceWrapper for Unnormalized<T> {
    type Inner = T;
}

unsafe impl<S> WrapPlace<S> for Unnormalized<S::Source>
where
    S: Subplace,
    S::Source: UnwrapUnnormalize,
    S::Target: UnwrapUnnormalize,
{
    type Wrapped = TransmutedSubplace<
        S,
        Unnormalized<S::Source>,
        <S::Target as UnwrapUnnormalize>::Unwrapped,
    >;

    fn wrap(subplace: S) -> Self::Wrapped {
        unsafe { TransmutedSubplace::new_unchecked(subplace) }
    }
}

unsafe impl<T> Matchable for Unnormalized<T>
where
    T: Matchable,
{
    const VARIANTS: &'static [&'static str] = T::VARIANTS;

    unsafe fn variant_at(ptr: *const Self) -> &'static str {
        unsafe { T::variant_at(ptr.cast()) }
    }
}

unsafe impl<const VARIANT: &'static str, T> HasVariant<VARIANT>
    for Unnormalized<T>
where
    T: HasVariant<VARIANT>,
    VariantType<T, VARIANT>: UnwrapUnnormalize,
{
    type VariantType =
        <VariantType<T, VARIANT> as UnwrapUnnormalize>::Unwrapped;
}
