# Wrapper types

Currently Rust does not automatically reborrow `Option<&mut T>` or similar
wrapper types. Conceptually, there is no reason why `&mut T` should be
reborrowable but `Option<&mut T>` should not: the only difference between the
two types is that one can also be null.

With the Reborrow trait, reborrowing wrapper types of exclusive references
becomes possible using blanket implementations.

```rust
impl<T: Reborrow> for Option<T> { /* ... */ }
```
