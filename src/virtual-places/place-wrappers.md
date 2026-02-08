# Place wrappers

> [!NOTE]
> This section is still fairly experimental. We definitely need place wrappers.
> But if we need additional syntax and how its trait looks like are still up in
> the air.
<!-- Make linkcheck happy -->
[!NOTE]: http://example.com

Until now, we covered just one of the motivations for *place wrappers*. The
primary use-case is to support the many container types of Rust. For example
`MaybeUninit<T>`, `Cell<T>`, `UnsafeCell<T>`, etc. All of these follow this
pattern:

> A `Container<Struct>` essentially is a struct with fields copied from
> `Struct`, but their type replaced with `Container<Field>`.

Phrasing this with the terms we have defined so far, a place wrapper is a
transparent generic type that "forwards" all subplaces of the generic and
changes their target types. So when `p: Container<T>`, we have that `p.proj` is
a valid place expression when `T.proj` is a valid projection, then `p.proj:
Container<proj::Target>`.

We encode this in the following trait:

```rust
pub unsafe trait PlaceWrapper<P>: HasPlace
where
    P: Projection<Source = Self::Target>,
{
    type WrappedProjection: Projection<Source = Self>;

    unsafe fn wrap_projection(proj: P) -> Self::WrappedProjection;
}
```

Note that this permits implementing `PlaceWrapper` selectively only for certain
projections.

## Examples

```rust
impl<T, P: Projection<Source = T>> PlaceWrapper<P> for MaybeUninit<T> {
    type WrappedProjection =
        TransparentProjection<P, MaybeUninit<T>, MaybeUninit<P::Target>>;

    fn wrap_projection(p: P) -> Self::WrappedProjection {
        TransparentProjection(p, PhantomData, PhantomData)
    }
}
```

```rust
pub struct TransparentProjection<P, Src, Tgt>(P, PhantomData<Src>, PhantomData<Tgt>);

impl<P: Projection, Src, Tgt> Projection for TransparentProjection<P, Src, Tgt> {
    type Source = Src;
    type Target = Tgt;

    fn offset(&self) -> usize {
        self.0.offset()
    }
}
```


TODO:

```rust
let cell: &Cell<Struct> = ...;
let value: Field = ...;
cell.field = value;
// desugars to:
<LocalPlace<Cell<Struct>> as PlaceWrite<
    <Cell<Struct> as PlaceWrapper>::WrappedProjection<
        field_proj!(Struct.field),
    >,
>>::write(
    &raw const cell,
    <Cell<Struct> as PlaceWrapper>::wrap_projection(
        <field_proj!(Struct.field)>::new(),
    ),
)
```
