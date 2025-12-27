# User-defined exclusive references

In some cases, users want to define their own custom reference types that have
equivalent semantics to Rust's exclusive `&mut T` references but cannot, for
whatever reason, be directly expressible using them. For example, an exclusive
reference to unaligned data or an exclusive reference to a part of a matrix
could not be expressed using `&mut T` references. In other cases, the
exclusivity of the reference may not be a guarantee but more of a suggestion:
eg. for mutable C++ references it may be a good idea to try use them as
exclusive, but exclusivity is not guaranteed and thus using `&mut T` instead of
a custom type would cause undefined behaviour.

```rust
#[derive(Reborrow)]

struct CustomMut<'a, T>(*mut T, PhantomData<&'a mut ()>);
```
