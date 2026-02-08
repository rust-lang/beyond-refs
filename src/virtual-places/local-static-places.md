# Local and static places

Having defined place wrappers, we can now define the language items for
supporting place operations on local variables and static variables.
Implementation wise, they have exactly the same `impl` blocks, so we just give
`LocalPlace`:

```rust
#[lang = "local_place"]
pub struct LocalPlace<T>(T);

impl<T> HasPlace for LocalPlace<T> {
    type Target = T;
}

unsafe impl<T, P: Projection<Source = T>> PlaceWrapper<P> for LocalPlace<T> {
    type WrappedProjection = TransparentProjection<P, LocalPlace<T>, P::Target>;

    unsafe fn wrap_projection(proj: P) -> Self::WrappedProjection {
        TransparentProjection(proj, PhantomData, PhantomData)
    }
}
```

There are several interesting things to note about this type:
- We require `T` to be sized. If Rust gains unsized locals again, then this
  could be relaxed.
- The target type of the wrapped projection is `P::Target`:
  `LocalPlace<Struct>` thus has all fields that `Struct` does.
- This previous point also affects the empty projection, which is no longer an
  "identity projection", but as a source has `LocalPlace<T>` and as the target
  has `T`.
- The compiler will involve this type directly in the desugaring of the place
  operations. This is the motivation behind using `P::Target` as the target.

## Notes

Authors of smart pointers can decide if they want their pointer to be able to
be created by borrowing a local or a static. Just implement `PlaceBorrow<P,
MySmartPtr<T>> for LocalPlace<T>`. This allows implementing something like a
`HeapOnlyRef<'_, T>`, which is statically known to point at the heap.
(the same cannot be done for stack-only refs, since futures and closures can be
stored on the heap and thus make locals also live there)
