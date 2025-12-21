# Method-based approach

Today, coercing exclusive references to shared references can be implemented in
user-land using a method-based approach:

```rust
trait CoerceShared {
    type Target: Copy;
    fn coerce_shared(&self) -> Self::Target;
}
```

This approach suffers from the downsides as
[method-based Reborrow does](./reborrow-methods.md). In addition, it is not
possible to fix the lifetime issues by simply not calling the `fn coerce_shared`
method as that would mean trying to use a `Self` type where `Self::Target` is
required.

The way to fix this is to define an `Into<Self::Target>` method that consumes
the source exclusive reference and produces a shared reference with the same
lifetime as a result. Then, instead of calling the `fn coerce_shared` method the
`fn into` method is called instead.

## Associated type or type argument

In general, there is no reason for why an associated type would be preferable
versus a type argument in the `CoerceShared` trait: especially with exclusive
reference collections it might make sense for a single reborrowable type to have
multiple `CoerceShared` targets. If the compiler automatically injects the
correct `fn coerce_shared` method call, then an associated type becomes
preferable.

The problem is that if the requirements for implementing `CoerceShared` are not
strict enough and a type argument is used, then the trait could become a vehicle
for generic automatic value coercion. For example:

```rust
struct Int(i64);

impl CoerceShared<i64> for Int {
    fn coerce_shared(&self) -> i64 {
        self.0
    }
}

impl CoerceShared<u64> for Int {
    fn coerce_shared(&self) -> u64 {
        self.0 as u64
    }
}

impl CoerceShared<i32> for Int {
    fn coerce_shared(&self) -> i32 {
        self.0 as i32
    }
}

impl CoerceShared<u32> for Int {
    fn coerce_shared(&self) -> u32 {
        self.0 as u32
    }
}

// ... and so on ...
```
