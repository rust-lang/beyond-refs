# Reborrow

Reborrowing is an action performed on exclusive references which creates a copy
of the source reference and marks it disabled for reads and writes. This retains
the exclusivity of the reference despite creating a copy, as only one copy can
be used at a time.

Today, true reborrowing is only available to Rust's exclusive `&mut T`
references. Going beyond references means enabling true reborrowing for
user-defined exclusive reference types by defining a `Reborrow` trait.

We want to make it possible for both of the following functions to compile: an
exclusive `&mut T` reference and a user-defined custom exclusive reference
`CustomMut<'_, u32>` should have equivalent semantics.

Example:

```rust
fn f(a: &mut u32) {
    f_x(a);
    f_y(a);
}

fn g(a: CustomMut<'_, u32>) {
    g_x(a);
    g_y(a);
}
```

## Use cases

- [User-defined exclusive references](./custom-mut.md)
- [Wrapper types](./wrapper-types.md)
- [Marker types](./marker-types.md)
- [Exclusive reference collections](./reference-collections.md)

## Approaches

The current approach to reborrowing in user-land is based on an explicit method.
The current work in the Reborrow traits lang experiment is based on a marker
trait.

- [Marker trait approach](./marker-trait.md)
- [Method-based approach](./methods.md)

## [CoerceShared](../coerce-shared/index.md)

Exclusive references call for a shared counterpart, into which an exclusive
reference can be coerced into. For Rust's references, this is `&T`. For
user-defined exclusive reference types, a shared counterpart is a second
user-defined type that is freely shareable (read: is `Copy`). Coercing a
user-defined exclusive reference into a shared reference type requires defining
a `CoerceShared` trait.

## Resources

[Tracking Issue for Reborrow trait lang experiment · Issue #145612 · rust-lang/rust](https://github.com/rust-lang/rust/issues/145612)

[Reborrow traits - Rust Project Goals](https://rust-lang.github.io/rust-project-goals/2025h2/autoreborrow-traits.html),
Jul 2025

[rfcs/text/0000-autoreborrow-traits.md at autoreborrow-traits · aapoalas/rfcs](https://github.com/aapoalas/rfcs/blob/autoreborrow-traits/text/0000-autoreborrow-traits.md),
May 2025

[Abusing reborrowing for fun, profit, and a safepoint garbage collector](https://github.com/aapoalas/abusing-reborrowing/tree/main)
(conference talk with examples), Feb 2025
