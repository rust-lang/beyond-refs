# Field representing types (FRTs)

{{#include cur.md}}

Field representing types, abbreviated FRTs, are a feature that allows
writing code that is generic over the fields of structs, enums,
tuples and variants of enums. They offer a limited form of
reflection, as Rust code can inspect the fields of its own types.

## Motivation

The most important application of FRTs is [Field
Projections](./field-projections.md). There they are one primitive way to
construct [Projections](./virtual-places/place-projections.md). FRTs can also
be used by normal functions that need to be generic over fields, but do not fit
into the field projection framework.

## Naming FRTs

FRTs are named using the `field_of!` macro. They are available for structs,
unions, tuples and variants of enums:

```rust
use std::field::field_of;

struct MyStruct {
    a: i32,
    b: u32,
}

type A = field_of!(MyStruct, a);
type B = field_of!(MyStruct, b);

union MyUnion {
    c: i32,
    d: u32,
}

type C = field_of!(MyUnion, c);
type D = field_of!(MyUnion, d);

type E = field_of!((i32, u32), 0);
type F = field_of!((i32, u32), 1);

enum MyEnum {
    Var1 { g: i32, h: u32 },
    Var2(i32, u32),
}

type G = field_of!(MyEnum, Var1.g);
type H = field_of!(MyEnum, Var1.h);
type I = field_of!(MyEnum, Var2.0);
type J = field_of!(MyEnum, Var2.1);
```

An FRT is visible when the field it represents is visible. In particular,
accessing the FRT of a private field from another module results in an error:

```rust
mod inner {
    pub struct MyStruct {
        a: i32,
        pub b: i32,
    }
}

type A = field_of!(inner::MyStruct, a); //~ ERROR: field `a` of struct `MyStruct` is private
type B = field_of!(inner::MyStruct, b);
```

## The `Field` trait

FRTs implement the `Field` trait, which exposes information about the field
that they represent:

```rust
pub unsafe trait Field: Sized {
    /// The type of the base where this field exists in.
    type Base;

    /// The type of the field.
    type Type;

    /// The offset of the field in bytes.
    const OFFSET: usize;
}
```

Note that this trait cannot be implemented manually, so only FRTs implement it.

For example, considering the following type definitions from above:

```rust
struct MyStruct {
    a: i32,
    b: u32,
}

union MyUnion {
    c: i32,
    d: u32,
}

enum MyEnum {
    Var1 { g: i32, h: u32 },
    Var2(i32, u32),
}
```

We have the following:
- `field_of!(MyStruct, a)` has:
  - `Base = MyStruct`,
  - `Type = i32`,
  - `OFFSET = offset_of!(MyStruct, a)`.
- `field_of!(MyUnion, c)` has:
  - `Base = MyUnion`,
  - `Type = i32`,
  - `OFFSET = 0`.
- `field_of!(MyEnum, Var1.g)` has:
  - `Base = MyEnum`,
  - `Type = i32`,
  - `OFFSET = offset_of!(MyEnum, Var1.g)`.

## Using FRTs

FRTs are usually used by APIs that wish to make an operation generically
available for each field of a struct, union, tuple or enum variant. To do so,
the API should introduce a generic parameter that implements the `Field` trait.
Since the trait cannot be implemented by non-FRTs, it ensures that only real
fields are allowed.

```rust
pub struct VolatileMut<'a, T: Copy> {
    ptr: *mut T,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T: Copy> VolatileMut<'a, T> {
    pub fn read_field<F: Field<Base = T>>(&self) -> F::Type {
        let ptr = self.ptr.offset();
        let ptr = unsafe { ptr.byte_add(F::OFFSET) };
        let ptr = ptr.cast::<F::Type>();
        unsafe { ptr.read_volatile() }
    }

    pub fn write_field<F: Field<Base = T>>(&mut self, value: F::Type) {
        let ptr = self.ptr.offset();
        let ptr = unsafe { ptr.byte_add(F::OFFSET) };
        let ptr = ptr.cast::<F::Type>();
        unsafe { ptr.write_volatile(value) };
    }
}
```

## Unresolved questions

- FRTs of structs, unions and tuples always exist in the type, but fields of
  enums are not necessarily accessible, as the value might not be of that
  variant. Probably need a separate trait to identify fields that always exist.
