# Dynamically sized types and metadata

In the next section we will define subplaces; these are represented by values
at runtime in order to support slices. Since subplace representing values must
always be valid subplaces, we need to obtain the metadata for the value behind
a smart pointer to query the length of a slice. For this reason, we introduce
our first place operation:

```rust
pub unsafe trait PlaceMetadata: HasPlace {
    unsafe fn metadata(this: *const Self) -> <Self::Target as Pointee>::Metadata;
}
```

It is allowed to read the pointer solely for the purpose of retrieving the
metadata. As with all operations, the exact safety requirements still need to
be fleshed out.

We will cover place operations much more generally later.

## Running example

Implementing it for `MyBox` looks like this:

```rust
unsafe impl<T: ?Sized> PlaceMetadata for MyBox<T> {
    unsafe fn metadata(this: *const Self) -> <Self::Target as Pointee>::Metadata {
        let ptr = unsafe { (*this).0 };
        ptr::metadata(ptr.as_ptr())
    }
}
```
