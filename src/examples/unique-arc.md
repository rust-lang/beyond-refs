# UniqueArc

`UniqueArc` is an `Arc` while it is known to be uniquely owned. Typically used for initialization,
after which it can be turned into a normal `Arc`.

```rust
// Safety: we're the unique pointer to this `T`.
// While this is live, the strong count is 0 so that we can give out other `Weak` pointers to this
// and prevent them from being upgraded.
pub struct UniqueArc<T>(Weak<T>);

// Supports: read, write, borrow fields as `&`, `&mut`

impl<T> UniqueArc<T> {
  pub fn new(x: T) -> Self { .. }
  pub fn downgrade(&self) -> Weak<T> { .. }
  // This sets the strong count to 1.
  pub fn into_arc(self) -> Arc<T> { .. }
}
```

[ACP 700](https://github.com/rust-lang/libs-team/issues/700) proposes a way to field-project
a `UniqueArc`: first we change how strong counts work. Instead of just a strong count, we cram two
counters into the `u64`, one for shared refs and one for unique refs, and update the rest of the
logic accordingly. Can't upgrade a weak pointer if there are any unique refs.
```rust
impl<T> UniqueArc<T> {
  // Checks the unique-ptr count.
  pub fn try_into_arc(self) -> Result<Arc<T>, NotUnique> { .. }
}

/// A subplace of a `UniqueArc`.
pub struct UniqueArcMap<T> {
  /// Pointer to the reference counts.
  header: NonNull<ArcHeader>,
  /// Pointer to the subplace we care about.
  val: NonNull<T>,
}

// Supports: read, write, borrow fields as `&`, `&mut`, reborrow fields as `UniqueArcMap`
```

The issue then is:
- If we don't give `UniqueArcMap` special borrowck behavior, then disjoint borrowing must be done
  with methods that split the borrow; that's sad.
- If we do give `UniqueArcMap` special borrowck behavior, then we can prevent multiple
  `@UniqueArcMap x.field` to the same fields. However, we can't get back a `Arc<T>` anymore:
```rust
let x: UniqueArc<Foo> = ...;
let field = @UniqueArcMap x.field;
// can't access `x.field` anymore
// in particular, can't call a method on `x` itself, since that may access `x.field`:
let arc = x.try_into_arc()?; // ERROR `x.field` is borrowed
```
