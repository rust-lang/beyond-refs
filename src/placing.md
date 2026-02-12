# Placing functions

This proposal provides an alternative syntax support for emplacement.
A new attribute `#[placing]` is introduced on the ADT type and function declaration site to signal
to the compiler that the type, function or closure is involved in emplacement and the appropriate
transformation is applied to support emplacement.

## Type annotation

This attribute at item location wraps the original ADT in a `MaybeUninit` and generate proper
destructors.

```rust
#[placing]
struct Data {
    value: Value,
}

// ... generates code equivalent to the following.
#[repr(transparent)]
struct Data(MaybeUninit<DataInner>);
struct DataInner {
    value: Value,
}

impl Drop for Data {
    fn drop(&mut self) {
        // Safety: generated `#[placing]` functions will never call this
        // unless initialisation is fully completed.
        unsafe {
            self.0.assume_init_drop();
        }
    }
}
```

## Constructor annotation

The same attribute but at function location demands that the return value to carry the `#[placing]`
attribute.
Given that, there is further code expansion so that these functions are emplacement constructors.

```rust
#[placing]
impl Data {
    #[placing]
    fn new() -> Data {
        Data {
            value: make_value(),
        }
    }
}
let _: Data = Data::new(); // OK
// ... generates code equivalent to the following.
impl Data {
    // The following function is generated exactly once
    unsafe fn new_uninit() -> Self {
        Self(MaybeUninit::uninit())
    }

    // This function initialises the value in-place.
    // Safety:
    // - This function shall not be called more than once.
    // - This function can only be called on value generated from `new_uninit`.
    unsafe fn new(&mut self) -> Data {
        let this = self.0.as_mut_ptr();
        unsafe {
            (&raw mut (*this).value).write(make_value());
        }
    }
}
```

## Method annotations

Inherent and trait associated methods within `impl` blocks annotated with `#[placing]` have
transparent access to fields with the help of the rewrite.

```rust
#[placing]
impl Data {
    fn get_value(&self) -> &Value {
        &self.value
    }
    fn set_value(&mut self, value: Value) {
        self.value = value;
    }
}
// ... generates code equivalent to the following
impl Data {
    fn get_value(&self) -> &Value {
        // Safety:
        // - This function can only be called after `new` is called.
        let this = unsafe { self.0.assume_init_ref() };
        // Here goes the rest of the original function:
        &this.value
    }
    fn set_value(&mut self, value: Value) {
        // Safety:
        // - This function can only be called after `new` is called.
        let this = unsafe { self.0.assume_init_mut() };
        // Here goes the rest of the original function:
        this.value = value;
    }
}
```

## Resources

[placing functions](https://blog.yoshuawuyts.com/placing-functions/), July 2025

[Placing Arguments](https://blog.yoshuawuyts.com/placing-arguments/), August 2025
