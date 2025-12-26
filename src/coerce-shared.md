# CoerceShared

{{#include stub.md}}

If exclusive references exist, then shared references are nearly always
necessary as well. Rust's own exclusive `&mut T` references automatically coerce
into shared `&T` references as necessary, and we want to enable this same
coercion for custom user-defined exclusive references as well. For this purpose
we define a `CoerceShared` trait.

## Use cases

To be fleshed out. All the same cases apply as for Reborrow.

Note that some custom reference cases might carry extra metadata (eg. a `usize`)
for exclusive references which then gets dropped out when coercing to shared.

## Approaches

The current approach to shared coercion in user-land is based on an explicit
method. The current work in the Reborrow traits lang experiment is based on a
marker trait.

- [Marker trait approach](./coerce-shared-marker-trait.md)
- [Method-based approach](./coerce-shared-methods.md)
