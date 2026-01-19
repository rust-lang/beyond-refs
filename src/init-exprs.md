# Init expressions

Init expression is an extension to Rust language syntax and trait ecosystem that prescribe a
in-place initialisation protocol.

## Traits

The core trait in the `core` standard library would be introduced with the following signature.

```rust
#![feature(try_trait_v2)]
/// Infallible initialisation
pub trait Init<Target> {
    fn init(self, slot: *mut Target);
}

/// Fallible initialisation
pub trait TryInit<Target> {
    type Residual;
    type Output: core::ops::Try<Output = (), Residual = Self::Residual>;
    fn try_init(self, slot: *mut Target) -> Self::Output;
}
```

## `init` functions

The core syntax extension is the `init` closure and `init` sugar on function signatures by
annotating which type fragment is the type of the destination place.

```rust
fn try_init_func(data: Data)
    -> Result<init MyStruct, Error> {
    //        ^~~~
    //        an implicit variable binding `out` as destination of emplacement ...
    // which has a type `MyStruct`.
    // It looks as if the following user variable declaration is implicitly
    // inserted at the beginning of the function call.
    //     let out: MyStruct;
    let processed_data = try_process(data)?;
    out.data = processed_data;
    // Similar to the out-pointer proposal, the emplacement variable `out`
    // has a drop obligation as soon as all the constituents are initialised.
    Ok(())
}

/// After lowering the type, which could technically only happen after HIR ...
/// one can inspect the type of the function and this is the actual function
/// "reified" type.
let _: fn(Data) -> impl TryInit<
    MyStruct,
    Residual = Result<Infallible, Error>,
    Output = Result<(), Error>
> = try_init_func;

fn try_init_func2(data: Data) -> Option<init(out) MyStruct> {
    out.data = process_opt(data)?;
    Some(())
}

let _: fn(Data) -> impl TryInit<
    MyStruct,
    Residual = Option<Infallible>,
    Output = Option<()>
> = try_init_func2;

// To emplace a value, one should invoke the emplacing constructor first,
// and then make use of the trait to perform the actual emplacement.
```

For `TryInit` functions or associated methods, the `TryInit::Residual` associated type is inferred
from the return value by replacing the type fragment with a prefix `init(..)` with `Infallible`.
For instance, `Result<init MyStruct, Error>` is desugared into a `Residual` type
`Result<Infallible, Error>` and `Option<init MyStruct>` into `Option<Infallible>`.
With this type rewrite, `TryInit` is able to work with `Try` types to support fallible emplacement
use cases.

---
**NOTE**

Following the Rust tradition of biasing towards explicitness, it has been proposed that there should
be a way to control how the output variable can be named.
The notable feature is that the sugar `init($ident)` indicates that a variable `$ident` will be made
available with the same type as the one following this syntax fragment.

Therefore, a user should specify which variable would be used as destination to which initialised
data should be written.

```rust
fn try_init_func(data: Data)
    -> Result<init(res) MyStruct, Error> {
    //             ^~~
    //             a variable binding `res` as destination of emplacement.
    let processed_data = try_process(data)?;
    res.data = processed_data;
    // Similar to the out-pointer proposal, the emplacement variable `res`
    // has a drop obligation as soon as all the constituents are initialised.
    Ok(())
}
```

---

## `init` expression

An important syntatical addition is to desugar ADT literals into `impl Init/TryInit` as well.

```rust
let initializer: impl Init<MyBiggerStruct> = init MyBiggerStruct {
    data <- init_data(details),
};

let initializer: impl TryInit<MyBiggerStruct> = init MyBiggerStruct {
    data <-? try_init_func(data),
};
// The `TryInit::Residual` type is inferred from `try_init_func`.
```

## `Box::new` with `impl Init`

`Box::<T>::new` would be adapted to allow accepting an `U` type instead where `U: Init<T>`,
given that we have an blanket `impl<T> Init<T> for T` and, in addition,
the selection of `impl Init` in general favours `impl Init` by user or environment over the blanket
implementation.

```rust
let _: Box<MyBiggerStruct> = Box::new(init MyBiggerStruct { .. }); // OK
let _: Box<MyBiggerStruct> = Box::new(MyBiggerStruct { .. }); // OK, but this is not emplacing

fn make_box<T: Init<MyBiggerStruct>>(emplace: T) -> Box<MyBiggerStruct> {
    Box::new(emplace)
    // ^ this is emplacing because `T: Init<MyBiggerStruct>` is favoured
}

// fallible case
fn make_fallible<T: TryInit<MyBiggerStruct, Residual = Result<Infallible, MyError>>>(emplace: T)
    -> Result<MyBiggerStruct, MyError>
where MyError: AllocError,
{
    Box::try_new_init(emplace) // this is emplacing with fallibility
}
```

## Resources

[Init expressions / In-place initialization](https://hackmd.io/@aliceryhl/BJutRcPblx), Dated Jun 7 2025
