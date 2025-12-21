# Marker trait approach

Coercing a custom user-defined exclusive reference into a user-defined shared
reference is a slightly involved affair: the two types are obviously going to be
different types, just like `&mut T` and `&T` are different, but they must also
be similar enough that a pure marker trait can relate the types to one another.

From a user's perspective, coercing an exclusive reference into shared is a
simple operation: just name the source exclusive reference type and the target
shared reference type.

This gives us the following trait definition:

```rust
trait CoerceShared<Target: Copy>: Reborrow {}
```

Its usage is done either manually through an `impl Trait` statement, or possibly
by deriving `CoerceShared` on the source exclusive reference type:

```rust
// impl Trait statement
#[derive(Reborrow)]
struct CustomMarker<'a>(PhantomData<&'a mut ()>);
struct CustomMarkerRef<'a>(PhantomData<&'a ()>);

impl<'a> CoerceShared<CustomMarkerRef<'a>> for CustomMarker<'a> {}

// derive macro
#[derive(Reborrow, CoerceShared(CustomMarkerRef))]
struct CustomMarker<'a>(PhantomData<&'a mut ()>);
struct CustomMarkerRef<'a>(PhantomData<&'a ()>);
```

As with [the Reborrow marker trait](./reborrow-marker-trait.md), some
limitations are placed on the trait although this time most of them are
expressed on the trait directly.

1. The type implementing `CoerceShared` must also implement `Reborrow`.

- Coercing an exclusive reference into a shared reference doesn't make sense if
  the source type is not an exclusive reference.

1. The result of the `CoerceShared` operation must be a `Copy` type.

- `CoerceShared` can be performed any number of times on the same value, always
  producing a byte-for-byte copy (ignoring any padding bytes). These results are
  thus copies of one another, so it must also be possible to perform
  `CoerceShared` once and produce copies of that result.

1. The result of the `CoerceShared` operation must have at least one lifetime.

- A lifetime-less type cannot contain the lifetime information that sound
  reborrowing relies upon.

1. The lifetime of the result must be equivalent to the source.

- If `CoerceShared` returns a lifetime that is always shorter than the source
  lifetime, then values deriving from the operation cannot be returned past it
  up the call stack. A longer lifetime is of course meaningless. Thus, the same
  lifetime is what we should get.

4. The target type must be relatable to the source type field-by-field.

- In order for the marker trait to be derivable by the compiler, its contents
  must be dericable from the source and target types. This is most reasonably
  performed field-by-field.

For exclusive reference types that have at most one data field and exactly one
lifetime, coercing into a shared reference type that has the same data field and
exactly one lifetime, the derivation of `CoerceShared` is trivial. For types
that have multiple fields and/or multiple lifetimes, the derivation becomes more
complicated.
