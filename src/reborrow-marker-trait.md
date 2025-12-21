# Marker trait approach

The current Reborrow traits lang experiment aims to produce a method-less
Reborrow trait. For exclusive reference semantics this is mostly trivial: the
concrete code generated for an exclusive reborrow is equivalent to that of
`Copy`, and only additional lifetime semantics must be considered on top.

This gives us a trivial, derivable `Reborrow` trait:

```rust
trait Reborrow {}
```

Its usage is performed through a derive macro:

```rust
#[derive(Reborrow)]
struct CustomMarker<'a>(PhantomData<&'a mut ()>);
```

There are some limitations that we wish to impose on types that derive (or
implement) this trait.

1. The type must not be `Clone` or `Copy`.

- Alternatively, blanket-implement `Reborrow for T: Copy`, but `Clone + !Copy`
  types cannot be `Reborrow`.
- This limitation is placed to avoid types that are both `Copy` and `Reborrow`,
  as that only makes sense if `Copy` types are considered a "base case" for
  recursive reborrowing.
- `Clone + !Copy` types generally only make sense if a corresponding `Drop`
  implementation exists, and reborrowing cannot be soundly combined with `Drop`.
  If a type is `Clone + !Copy + !Drop` then technically it can be soundly
  reborrowed, but it's unlikely that the type itself makes overmuch sense (it is
  by definition a cloneable, ie. shareable, type with exclusive reference
  semantics).

1. The type must not have a `Drop` implementation.

- Reborrowable types can have multiple owned copies in existence, and each one
  of these would need to call `Drop` (as we cannot know if we have the only copy
  or not). This is effectively guaranteed to end up causing a double-free.

1. The type must have at least one lifetime.

- Alternatively, if `Copy` types blanket-implement `Reborrow` then this
  limitation cannot be placed, but manually deriving `Reborrow` on a
  lifetime-less type should be an error.
- This limitation is placed simply because a lifetime-less type cannot contain
  the lifetime information that sound reborrowing relies upon.

1. The result of a `Reborrow` operation is simply the type itself, including the
   same lifetime.

- Reborrowing should produce a copy, not eg. a field or a new struct consisting
  of a subset of the type's fields.
- If `Reborrow` returns a lifetime that is always shorter than the source
  lifetime, then values deriving from the operation cannot be returned past it
  up the call stack. A longer lifetime is of course meaningless. Thus, the same
  lifetime is what we should get.

## Derivation of the `Reborrow` operation

Since the marker trait approach has no explicit `fn reborrow` method, the
compiler must derive the correct operation for `Reborrow`. When exactly one
lifetime exists on the type, that derivation is trivial: it is simply a `Copy`
and an exclusive reborrow of the singular lifetime. Though, this too can be
questionable:

```rust
#[derive(Reborrow)]
struct Bad<'a>(&'a ());
```

Is the above type actually an exclusive reference? Is deriving `Reborrow` on it
an error? I think it should be, but it's a little hard to exactly say why:
arguably it's because reborrowing (re)asserts exclusive access while this type,
by definition, does not have exclusive access to anywhere. But maybe this is
created using a `&mut ()` in which case it sort of does carry exclusive access,
it just cannot re-assert it.

If there are multiple lifetimes, then deriving `Reborrow` becomes more
complicated. Consider the following type:

```rust
#[derive(Reborrow)]
struct Two<'a, 'b>(&'a (), &'b mut ());
```

This type should only exclusively reborrow the second reference.

These questions become a step more complicated once we give up on using Rust
references and go into the world of custom types:

```rust
#[derive(Reborrow)]
struct MaybeBad<'a>(PhantomData<&'a ()>);
```

This type may or may not be bad: we simply have no idea. Whether the borrow
inside of `PhantomData` is `&mut ()` or `&()` has no effect on whether or not
this type carries an "exclusive lifetime" or not.

This issue seems to call for a new kind of marker type:

```rust
/// Marker type for exclusive reference semantics.
#[derive(Reborrow)]
struct PhantomExclusive<'a>; // explicitly no content; this is purposefully bivariant.

impl<'a> PhantomExclusive<'a> {
    /// Capture an exclusive reference into a PhantomExclusive.
    fn from_mut<T>(_: &'a mut T) -> Self {
        Self
    }

    /// Create a new unbound PhantomExclusive.
    ///
    /// # Safety
    ///
    /// * The caller must ensure that only one PhantomExclusive is created for
    ///   whatever that they're tracking.
    unsafe new() -> Self {
     Self
    }
}
```

The compiler would track `PhantomExclusive` as an exclusive reference without
the pointer bits. Our custom type would then be:

```rust
#[derive(Reborrow)]
struct Good<'a>(PhantomData<&'a ()>, PhantomExclusive<'a>);
```

The first `PhantomData` is there to establish variance while the
`PhantomExclusive` is included to ensure `'a` is an "exclusive lifetime".

## Recursive nature of deriving `Reborrow`

The derivation of `Reborrow` (and `CoerceShared`) has a recursive nature: we can
group up individual exclusive references (be they `&mut T` or `CustomMut<'_, T>`
or `CustomMarker<'_>`) into a single struct and derive `Reborrow` on it. This
derivation is done on the fields of the type, and when a field is found to be
`Reborrow` then that field's reborrow operation becomes a part of the larger
whole's operation.

This has some complexities regarding what are the "bottom/base cases" of the
recursion.

- `&'a mut T` bottoms out and performs a reborrow on `'a`
- `CustomMut<'a, T>` can be assumed to bottom out and reborrow on `'a`.
- `Two<'a, 'b>` needs to be checked: does it reborrow both `'a` and `'b` or only
  one of them?
- `PhantomData<&'a ()>` bottoms out and DOES NOT perform a reborrow on `'a`:
  this is a `Copy` marker.
- `PhantomExclusive<'a>` bottoms out and performs a reborrow on `'a`.
