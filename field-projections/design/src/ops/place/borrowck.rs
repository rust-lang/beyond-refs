use std::marker::PhantomData;

use macros::sealed;

#[sealed]
pub trait Timing {}

pub struct Instant;
pub struct Lifetime<'a>(PhantomData<&'a ()>);
pub struct UntilDrop;

#[sealed]
impl Timing for Instant {}
#[sealed]
impl Timing for Lifetime<'_> {}
#[sealed]
impl Timing for UntilDrop {}

pub enum AccessKind {
    Shared,
    Exclusive,
    Untracked,
}
