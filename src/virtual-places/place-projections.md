# Subplaces and projections

Places (defined as place expressions from the previous section) can have
*subplaces*. For example given a place `p` with the type `Struct` that has a
field called `field`, the place `p.field` is a *subplace* of `p`. The
characterizing definition of a subplace is that it must occupy a subrange of
the allocation from its parent place. So given `Box<T>`, the `T` is **not** a
subplace, since its allocation is distinct from the `Box` (which might live on
the stack). We are interested in subplaces, since place operations can take
place "at the same time" when their subplaces are disjoint.

Projections codify subplaces in the type system of Rust; we later will use
generics bounded on this trait to define operations *generic over all
projections*. Since we also want to support dynamic projections like
`array[idx]`, values of projection types can also carry information. A
projection is represented by the following trait:

```rust
pub unsafe trait Projection: Sized + Copy {
    type Source: ?Sized;
    type Target: ?Sized;

    fn offset(
        self,
        metadata: <Self::Source as Pointee>::Metadata,
    ) -> (usize, <Self::Target as Pointee>::Metadata);
}
```

An instance `p: P` of a projection `P: Projection` stores the following
information:
- for a specific place `q` of type `P::Source` with metadata `meta`, there is a
  subplace of type `P::Target` living at byte index `p.offset(meta).0` and that
  subplace has metadata `p.offset(meta).1`.

The important property of a projection is that it always remains in-bounds of
the allocation of `P::Source`, not involving any dereferences. This is
required, since pointer might not point at memory valid for reads.

## Kinds of projections

### Empty projection

The empty projection has no offset and doesn't change the type and metadata:

```rust
pub struct EmptyProjection<T: ?Sized>(PhantomData<T>);

unsafe impl<T: ?Sized> Projection for EmptyProjection<T> {
    type Source = T;
    type Target = T;

    fn offset(
        self,
        metadata: <Self::Source as Pointee>::Metadata,
    ) -> (usize, <Self::Target as Pointee>::Metadata) {
        (0, metadata)
    }
}

impl<T: ?Sized> Default for EmptyProjection<T> { /* ... */ }
```

### Field projection

> [!CAUTION]
> This term has potential for confusion, since it bears the same name as the
> overarching concept of field projections. (FIXME?)
<!-- Make linkcheck happy -->
[!CAUTION]: http://example.com

Given a struct `Struct`, we can talk about the projection to any of its fields.
So we essentially have a mapping from [field representing
types](../field-representing-types.md) to projections:

```rust
pub struct FieldProjection<T: ?Sized, F: Field<Base = T>>(PhantomData<T>, PhantomData<F>);

unsafe impl<T: ?Sized, F: Field<Base = T>> Projection for FieldProjection {
    type Source = T;
    type Target = F::Type;

    fn offset(
        self,
        metadata: <Self::Source as Pointee>::Metadata,
    ) -> (usize, <Self::Target as Pointee>::Metadata) {
        // need some compiler magic, or reflection, or specialization:
        if (F::Type: Sized) {
            (F::OFFSET, ())
        } else {
            (F::offset(metadata), metadata)
        }
    }
}

impl<T: ?Sized> Default for FieldProjection<T> { /* ... */ }

macro_rules! field_proj {
    ($struct:ty, $field:ident) => {
        $crate::FieldProjection::<$struct, ::core::field::field_of!($struct, $field)>
    }
}
```

The `offset` function is generally constant and statically known for field
projections. However, for dynamically sized structs, the offset of the last
field (the one that has dynamic size) depends on the alignment of the concrete
instance. In that case, `offset` depends on the concrete instance of the
projection. To create such a projection, we need to obtain the metadata of the
place, so only smart pointers that implement `PlaceMetadata` support these.

Field projections also exist for enums and unions, however they require unsafe,
since they aren't always valid to apply.

### Index projections

Arrays and slices have many subplaces that can be accessed by indexing:

```rust
pub struct IndexProjection<T: ?Sized + CanBeIndexedBikeshed>(usize, PhantomData<T>);

// implemented for [T] and [T; N]
#[sealed]
pub trait CanBeIndexedBikeshed {
    type Element;
}

impl<T: ?Sized> IndexProjection<T> {
    pub fn new(offset: usize) -> Self {
        Self(offset, PhantomData)
    }
}

impl<T: ?Sized + CanBeIndexedBikeshed> Projection for IndexProjection {
    type Source = T;
    type Target = T::Element;

    fn offset(
        self,
        metadata: <Self::Source as Pointee>::Metadata,
    ) -> (usize, <Self::Target as Pointee>::Metadata) {
        (self.0, ())
    }
}
```

### Composing projections

Since projections never leave the allocation, we can combine two projections:

```rust
#[derive(Copy, Clone)]
pub struct ComposedProjection<P, Q>(pub P, pub Q);

unsafe impl<P, Q> Projection for ComposedProjection<P, Q>
where
    P: Projection,
    Q: Projection<Source = P::Target>,
{
    type Source = P::Source;
    type Target = Q::Target;

    fn offset(
        self,
        metadata: <Self::Source as Pointee>::Metadata,
    ) -> (usize, <Self::Target as Pointee>::Metadata) {
        let (offset, metadata) = self.0.offset(metadata);
        let (offset2, metadata) = self.1.offset(metadata);
        (offset + offset2, metadata)
    }
}
```

In order to combine two projections `p` and `q`, the subplace that is
represented by `p` must allow the projection represented by `q`.

## Projection notation and desugaring

In the remaining sections we will often write `place.proj` for a place
expression made up of a `place` and a compatible projection `proj`. In this
case, `proj` represents both the type that implements `Projection` and the
concrete instance of that type. We also expect the reader to understand that
`proj` is *maximal*, so `place` either is a local/static variable, or a
dereference (but not a field access or index operation). Also note that `proj`
can be `EmptyProjection`, so we can also cover non-projecting place expressions
with writing `place.proj`.

This allows us to abstract over "taking a subplace" and not have to worry about
handling all the specific cases of subplaces.
