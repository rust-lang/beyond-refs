use std::{
    marker::PhantomInvariant,
    ptr::Pointee,
};

use crate::{
    place::{
        Field,
        Subplace,
    },
    ptr::Metadata,
};

pub struct TransmutedSubplace<Sub, Source: ?Sized, Target: ?Sized> {
    subplace: Sub,
    _source: PhantomInvariant<Source>,
    _target: PhantomInvariant<Target>,
}

impl<Sub: Default, Source: ?Sized, Target: ?Sized> Default
    for TransmutedSubplace<Sub, Source, Target>
{
    fn default() -> Self {
        Self {
            subplace: Default::default(),
            _source: Default::default(),
            _target: Default::default(),
        }
    }
}

unsafe impl<Sub, Source, Target> Subplace
    for TransmutedSubplace<Sub, Source, Target>
where
    Sub: Subplace,
    Source: ?Sized + Pointee<Metadata = Metadata<Sub::Source>>,
    Target: ?Sized + Pointee<Metadata = Metadata<Sub::Target>>,
{
    type Source = Source;

    type Target = Target;

    fn offset(
        self,
        metadata: Metadata<Self::Source>,
    ) -> (usize, Metadata<Self::Target>) {
        self.subplace.offset(metadata)
    }
}

impl<Sub, Source, Target> TransmutedSubplace<Sub, Source, Target>
where
    Sub: Subplace,
    Source: ?Sized,
    Target: ?Sized,
{
    pub unsafe fn new_unchecked(p: Sub) -> Self {
        Self {
            subplace: p,
            _source: PhantomInvariant::new(),
            _target: PhantomInvariant::new(),
        }
    }
}

pub struct TransmutedField<F, Source: ?Sized, Target: ?Sized>(
    TransmutedSubplace<F, Source, Target>,
);

impl<F: Default, Source: ?Sized, Target: ?Sized> Default
    for TransmutedField<F, Source, Target>
{
    fn default() -> Self {
        Self(Default::default())
    }
}

unsafe impl<F, Source, Target> Subplace for TransmutedField<F, Source, Target>
where
    F: Field,
    Source: ?Sized + Pointee<Metadata = Metadata<F::Source>>,
    Target: ?Sized + Pointee<Metadata = Metadata<F::Target>>,
{
    type Source = Source;
    type Target = Target;

    fn offset(
        self,
        metadata: Metadata<Self::Source>,
    ) -> (usize, Metadata<Self::Target>) {
        self.0.subplace.offset(metadata)
    }
}

unsafe impl<F, Source, Target> Field for TransmutedField<F, Source, Target>
where
    F: Field,
    Source: ?Sized + Pointee<Metadata = Metadata<F::Source>>,
    Target: ?Sized + Pointee<Metadata = Metadata<F::Target>>,
{
    const NAME: &'static str = F::NAME;
}
