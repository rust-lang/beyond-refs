# Canonical borrows

Supporting canonical borrows is a simple exercise of writing down the correct
trait:

```rust
pub trait CanonicalPlaceBorrow:
    HasPlace
    + for<P: Projection<Source = Self::Target>> PlaceBorrow<P, Self::Output<P>>
    + for<P: Projection<Source = Self::Target>> PlaceDeref<P>
{
    type Output<P: Projection<Source = Self::Target>>: HasPlace<Target = P::Target>;
}
```

The `for<P...>` syntax is not available to Rust and not part of this proposal.
It expresses that this trait needs compiler support in order to require that
bound to be true for all possible projection types.

The desugaring of `@place` is the following:
- if `place == local.proj`, then we desugar it to:
  ```rust
  @<<typeof(local) as CanonicalReborrow>::Output<projection!(proj)>> place
  ```
- if `place == (*place').proj`, then we desugar it to:
  ```rust
  @<<typeof(place') as CanonicalReborrow>::Output<projection!(proj)>> place
  ```

Due to the implicit compiler bound, this expression can never error due to a
missing `PlaceBorrow` or `PlaceDeref` implementation.
