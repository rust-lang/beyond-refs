# Exotic Examples for Field Projections

This is a list of ideas that require more work on our proposal to integrate, these have not been fleshed out yet and might not make it into the final design. Nevertheless, they enable interesting use-cases and we'll try to integrate them.

## `FileSlice<T>` (Alice)

- Have a type `FileSlice<T>` type that points at a `T: FromBytes` inside of a file with a file-offset that's `u64`.
    - Projections require that `P::Target: FromBytes` (we already support this with a `where` bound on the `PlaceBorrow` impl).
    - We also want to use `PlaceRead` to perform a [`read_at`](https://doc.rust-lang.org/stable/std/os/unix/fs/trait.FileExt.html#tymethod.read_at) call and extract the contents of a `T`. But that call can not only fail, but also not read enough bytes, because the file is too short. Our current `PlaceRead::read` function must return a `<Self as HasPlace>::Target`, which we want to be `T` for `FileSlice` in order to support projecting to subfields.
- Needs to have custom projections that allow for `u64` instead of `usize` offsets.

## `HashMap<K, V>` (Alice & Benno)

- With custom projections, we could allow `map["key"] = value;` to mean `map.insert("key", value);`.
- Needs to have custom projections that allow for `&'static str` (or any `Sized` value in general) instead of `usize`.
- We would also need to coordinate with `Index[Mut]`, since we're now generalizing it too (maybe we need to do that already).
