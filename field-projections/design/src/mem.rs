use std::mem::MaybeUninit;

use crate::{
    ops::place::{
        PlaceWrapper,
        WrapPlace,
    },
    place::{
        Subplace,
        TransmutedSubplace,
    },
};

impl<T> PlaceWrapper for MaybeUninit<T> {
    type Inner = T;
}

unsafe impl<S> WrapPlace<S> for MaybeUninit<S::Source>
where
    S: Subplace,
    S::Source: Sized,
    S::Target: Sized,
{
    type Wrapped =
        TransmutedSubplace<S, MaybeUninit<S::Source>, MaybeUninit<S::Target>>;

    fn wrap(subplace: S) -> Self::Wrapped {
        unsafe { TransmutedSubplace::new_unchecked(subplace) }
    }
}
