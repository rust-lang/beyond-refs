use design::{
    ops::place::{
        PlaceWrapper,
        WrapPlace,
    },
    place::{
        Subplace,
        TransmutedSubplace,
    },
};

#[repr(transparent)]
pub struct Untrusted<T: ?Sized>(T);

impl<T: ?Sized> PlaceWrapper for Untrusted<T> {
    type Inner = T;
}

unsafe impl<S: Subplace> WrapPlace<S> for Untrusted<S::Source> {
    type Wrapped =
        TransmutedSubplace<S, Untrusted<S::Source>, Untrusted<S::Target>>;

    fn wrap(subplace: S) -> Self::Wrapped {
        unsafe { TransmutedSubplace::new_unchecked(subplace) }
    }
}
