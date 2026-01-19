# Summary

_This is the document archived from [Init expressions / In-place initialization](https://hackmd.io/@aliceryhl/BJutRcPblx) authored by Alice Ryhl._

Introduce a new trait `Init` for initializers. Initializers are essentially
functions that write a value to the place provided to it. Along with the trait,
we also introduce:

* A second trait `PinInit` for handling the case where the value is pinned after
  initialization.
* A new syntax `init {expr}` that creates an initializer. This lets you compose
  initializers, and it is essentially a type of closure.
* A new mechanism so that dyn compatible traits can have methods returning `impl
  Trait` (including async fns).

# Motivation

This document outlines a mechanism for in-place initialization. I believe it
solves the following issues:

* Stack overflow when creating values on the heap
* Constructors returning pinned values
* C++ move constructors
* Async fn in dyn Trait

It works both with values that require pinning, and values that do not.

## Stack overflow when creating values on the heap

If you attempt to allocate a box with `Box::new(value)`, then the compiler will
often generate code that performs the following steps:

1. Construct `value` on the stack.
2. Allocate memory on the heap.
3. Move `value` from the stack to the heap.

If `value` is large, then this may result in a stack overflow during the first
step. The stack overflow can be avoided if the compiler instead performed these
steps:

1. Allocate memory on the heap.
2. Construct `value` directly on the heap.

However, the compiler is often unable to perform this optimization because the
straight-line execution of `Box::new(value)` is that `value` should be created
before allocating memory.

In-place initialization provides a convenient alternative to `Box::new(value)`
where the straight-line execution is to first allocate the memory and then
construct the value directly on the heap. This means that the behavior we want
does not require any optimization.

## Constructors returning pinned values

You might want to implement a linked list like this:

```rust
struct Entry {
    next: *mut Entry,
    prev: *mut Entry,
}

struct List {
    // The list has an extra entry `dummy` that does not correspond to an actual
    // element. Here, `dummy.next` is the first actual element and `dummy.prev`
    // is the last actual element.
    dummy: Entry,
}

impl List {
    fn new() -> List {
        List {
            dummy: Entry {
                next: < what goes here? >,
                prev: < what goes here? >,
            }
        }
    }

    fn push_front(&mut self, entry: *mut Entry) {
        entry.next = self.dummy.next;
        entry.prev = self.dummy;
        self.dummy.next = entry;
    }
}
```

Here, `List::new()` needs to initialize the pointers to `dummy` so that
`push_front` has the correct behavior when pushing to an empty list. But that is
tricky to do. This implementation does not work:

```rust
impl List {
    pub fn new() -> List {
        let mut me = List { ... };
        me.dummy.prev = &mut self.dummy;
        me.dummy.next = &mut self.dummy;

        // Oops, this changes the address of `me`.
        me
    }
}
```

In-place initialization makes it possible to write constructors like this, as
the final address is known during initialization when using in-place
initialization.

This implementation of linked list is extremely common in the Linux kernel
because it avoids a branch for the empty list case in `push`. Linux uses it
inside most data structures. For example, the Linux kernel mutex uses this type
of linked list for the queue of waiters.

## C++ move constructors

Most values in C++ are not trivially relocatable. This means that you must run
user-defined code (the move constructor) to move them from one address to
another. It would be nice if we could move a C++ value using code along these
lines:

```rust
let new_value = old_value.take();
```

where `take()` creates an initializer that calls the C++ move constructor.
In-place initialization makes code like this possible because it lets
`take()` know what the address of `new_value` is.

Calling the move constructor still requires an explicit call to `take()`, but
Rust generally prefers for such operations to be explicit anyway, so I don't see
this as a downside.

## Async fn in dyn Trait

Given the following trait:

```rust
trait MyTrait {
    async fn foo(&self, arg1: i32, arg2: &str);
}
```

we would like `MyTrait` to be dyn compatible. It is not dyn compatible today
because `<dyn MyTrait>::foo` returns a future of unknown size, and the caller
needs to know the size.

There are various proposal for how to solve this, the most prominent being
autoboxing where the compiler generates a `<dyn MyTrait>::foo` that returns
`Pin<Box<dyn Future>>`. Boxing the future solves the issue with the size being
unknown, but autoboxing has several downsides:

* The allocation strategy is not configurable. I.e., you could not implement
  your own logic that conditionally stores the future on the stack or the heap
  depending on the size.
* It might require new syntax such as `my_trait.foo().box`.
* It introduces implicit calls to `Box::new` in the language itself, which many
  community members including myself believe to be against the design goals of
  Rust.

In-place initialization provides a solution that lets you call an async function
`<dyn MyTrait>::foo` without any of the above downsides.

Note that it generalizes to

```rust
trait MyTrait {
    fn foo(&self, arg1: i32, arg2: String) -> impl MyOtherTrait;
}
```

as long as `MyOtherTrait` is dyn-compatible.

# Guide-level explanation

The standard library has two traits that looks like this:

```rust
/// Initializers that can create values using in-place initialization.
unsafe trait PinInit<T> {
    type Error;

    unsafe fn init(self, slot: *mut T) -> Result<(), Self::Error>;
}

/// Initializers that can create values using in-place initialization.
///
/// Note that all initializers also implement `PinInit`, since you can always
/// pin a value immediately after creating it.
trait Init<T>: PinInit<T> { }
```

The real traits are slightly more complex to support `T: ?Sized`, but are
otherwise equivalent to the above. See the reference-level explanation for the
full traits.

The standard library also comes with the following blanket implementations:

```rust
impl<T> PinInit<T> for T {
    type Error = Infallible;

    fn init(self, slot: *mut T) -> Result<(), Infallible> {
        slot.write(self);
        Ok(())
    }
}

impl<T,E> PinInit<T> for Result<T,E> {
    type Error = E;

    fn init(self, slot: *mut T) -> Result<(), E> {
        slot.write(self?);
        Ok(())
    }
}

impl<T> Init<T> for T {}
impl<T,E> Init<T> for Result<T,E> {}
```

## Using initializers

Using the above declarations, many constructors become possible. For example, we
may write these for `Box`.

```rust
impl<T> Box<T> {
    pub fn try_init<E>(i: impl Init<T, Error = E>) -> Result<Box<T>, E> {
        let mut me = Box::<T>::new_uninit();
        i.init(me.as_mut_ptr())?;
        Ok(me.assume_init())
    }

    pub fn init(i: impl Init<T, Error = Infallible>) -> Box<T> {
        Box::try_init(i).unwrap()
    }

    pub fn try_pinit<E>(i: impl PinInit<T, Error = E>) -> Result<Pin<Box<T>>, E> {
        let mut me = Box::<T>::new_uninit();
        i.init(me.as_mut_ptr())?;
        Ok(Pin::from(me.assume_init()))
    }

    pub fn pinit(i: impl Init<T, Error = Infallible>) -> Pin<Box<T>> {
        Box::try_pinit(i).unwrap()
    }

}
```

Other possible cases are constructors for `Arc`, or methods such as
`Vec::push_emplace`.

Initializers may also be used on the stack with the macros `init!` or `pinit!`.

```rust
let value = init!(my_initializer);
let value2 = pinit!(my_pin_initializer);
```

Note that `pinit!` internally works like the `pin!` macro and returns a
`Pin<&mut T>`. The `init!` macro returns a `T`.

## The `init` syntax

A new syntax similar to closures is introduced. It allows you to compose
initializers. The syntax is `init` followed by an expression. As an example,

```rust
init MyStruct {
    field_1: i1,
    field_2: i2,
}
```

desugars to an initializer that creates a `MyStruct` by first running `i1` to
create `field_1` and then running `i2` to create `field_2`. Arrays are also
supported:

```rust
init [i; 1_000_000]
```

desugars to an initializer that evaluates `i` one million times.

This logic does not work recursively. To get recursive treatment, you need
to use `init` multiple times.

```rust
init MyStruct {
    foo: init (17, init MyTupleStruct(i1)),
    bar: init MySecondStruct {
        baz: init [i2; 10],
        raw_value: my_value,
    }
}
```

This ultimately treats `17`, `i1`, `i2`, and `my_value` as the initializers to
run. Note that due to the blanket impls that makes any type an initializer for
itself, using `17` and `my_value` works seamlessly even if they're just the
value to write.

All initializers in an `init` expression must have the same error type. If initialization of a field fails, then the previously initialized fields are dropped and the initialization of the struct fails.

The initializers are run in the order they appear in the `init` expression.
Because of that, previous fields may be accessed by name in the expression for
creating the next initializer. For example, you can create a field that holds
the same value twice like this:

```rust
init MyStruct {
    foo: my_expensive_initializer(),
    bar: foo.clone(),
}
```

You may also use an underscore to run additional code during the initializer:

```rust
init MyStruct {
    foo: 12,
    _: {
        println!("Initialized foo to {foo}.");
    },
}
```

The RHS of an underscore must evaluate to an initializer for `()`. They may be used to modify previous fields, or to fail the initializer by returning an error.

```rust
struct MyStruct {
    inner: bindgen::some_c_struct,
}

impl MyStruct {
    fn new(name: &str) -> impl Init<MyStruct, Error> {
        init MyStruct {
            inner: unsafe { core::init::zeroed() },
            _: {
                let ret = unsafe {
                    bindgen::init_some_c_string(&mut inner, name)
                };

                if ret < 0 {
                    Err(Error::from_errno(ret))
                } else {
                    // Result<(), Error> is an initializer
                    // for ().
                    Ok(())
                }
            },
        }
    }   
}
```

### Pinned fields

Normally, all initializers using in an `init` expression must implement the
`Init` trait. However, this can be relaxed to only requiring `PinInit` using the
`#[pin]` annotation.

```rust
struct MyStruct {
    f1: String,
    #[pin]
    f2: MyPinnedType,
}
```

In this case, `init MyStruct { f1: i1, f2: i2 }` requires `i1` to implement
`Init<String>`, but the requirements for `i2` are relaxed to only require
`PinInit<MyPinnedType>`.

The opaque type returned by an `init` expression always implements `PinInit`. It
implements `Init` if and only if it is composed using only initializers that
implement `Init`.

Whenever `#[pin]` is present on at least one field, implementations of `Drop`
need to use the signature `fn drop(self: Pin<&mut Self>)` instead of the normal
signature. The compiler additionally allow you to obtain an `Pin<&mut Field>`
given an `Pin<&mut MyStruct>` when a field is annotated with `#[pin]`.
Similarly, you can obtain `&mut Field` given `Pin<&mut MyStruct>` when a field
is not annotated with `#[pin]`. The compiler-generated impl for `Unpin` needs to
be adjusted to match.

Or in other words, we make `pin-project` into a language feature over an
edition.

## Impl trait in dyn trait

If you have a trait such as

```rust
trait MyTrait {
    fn foo(&self, arg1: i32, arg2: &str) -> impl MyOtherTrait;
}
```

then `MyTrait` is dyn-compatible with `<dyn MyTrait>::foo` being a
compiler-generated method that returns an opaque type that implements `Init<dyn MyOtherTrait>`.

This allows you to do things such as:

```rust
trait MyTrait {
    async fn call(&self);
}

async fn my_fn(value: &dyn MyTrait) {
    Box::pinit(value.call()).await;
}
```

This is nice since it is explicit that you're boxing the future returned by
`call` so that you can await it.

Of course, `value.call().await` would not work as you don't have a future.
However, we should be able to emit good error messages for this case suggesting
that you wrap it in a box.

One advantage of this design is the flexibility with regards to boxing. The above example shows that we can box the future, but you could also easily support storing it on the stack if the future is small, and only allocate a box for large futures.

Note that `dyn MyTrait` does _not_ implement `MyTrait` under this design, as `<dyn MyTrait>::call` does not return a future.

# Reference-level explanation

We add the following two traits to `core::init`.

```rust
/// # Safety
///
/// Implementers must ensure that if `init` returns `Ok(metadata)`, then
/// `core::ptr::from_raw_parts_mut(slot, metadata)` must reference a valid
/// value owned by the caller. Furthermore, the layout returned by using
/// `size_of` and `align_of` on this pointer must match what `Self::layout()`
/// returns exactly.
unsafe trait PinInit<T: ?Sized + Pointee> {
    type Error;

    /// Writes a valid value of type `T` to `slot` or fails.
    ///
    /// If this call returns `Ok`, then `slot` is guaranteed to contain a valid
    /// value of type `T`. If `T` is unsized, then `slot` may be combined with
    /// the metadata to obtain a valid pointer to the value.
    ///
    /// Note that `slot` should be thought of as a `*mut T`. A unit type is used
    /// so that the pointer is thin even if `T` is unsized.
    ///
    /// # Safety
    ///
    /// The caller must provide a pointer that references a location that `init`
    /// may write to, and the location must have at least the size and alignment
    /// specified by `PinInit::layout`.
    ///
    /// If this call returns `Ok` and the initializer does not implement
    /// `Init<T>`, then `slot` contains a pinned value, and the caller must
    /// respect the usual pinning requirements for `slot`.
    unsafe fn init(self, slot: *mut ()) -> Result<T::Metadata, Self::Error>;

    /// The layout needed by this initializer.
    fn layout(&self) -> Layout;
}

/// Indicates that values created by this initializer do not need to be pinned.
///
/// # Safety
///
/// Implementers must ensure that the implementation of `init()` does not rely
/// on the value being pinned.
unsafe trait Init<T: ?Sized + Pointee>: PinInit<T> {}
```

The standard library also comes with the following implementations:

```rust
unsafe impl<T> PinInit<T> for T {
    type Error = Infallible;

    fn init(self, slot: *mut T) -> Result<(), Infallible> {
        slot.write(self);
        Ok(())
    }

    fn layout(&self) -> Layout {
        Layout::new::<T>()
    }
}

// SAFETY: Even if `T: !Unpin`, this impl can only be used if we have `T` by
// ownership which implies that the value has not yet been pinned.
unsafe impl<T> Init<T> for T {}

impl<T,E> PinInit<T> for Result<T,E> {
    type Error = E;

    fn init(self, slot: *mut ()) -> Result<(), E> {
        slot.write(self?);
        Ok(())
    }

    fn layout(&self) -> Layout {
        Layout::new::<T>()
    }
}

impl<T,E> Init<T> for Result<T,E> {}
```

## Creating initializers

The `init` expression is parsed into one of the following cases:

### Struct syntax

If it matches `init StructName { ... }`, then the struct expression is parsed to
obtain an ordered list of fields. Fields can take the following forms:

* `field: rhs`
* `name @ field: rhs`
* `_: rhs`

This results in an initializer whose `init` function runs the initializers in
the order they are listed. That is, to construct a field, it will call
`PinInit::init(rhs, <ptr to field>)`.

When evaluating `rhs`, previously initialized fields are in scope. The
`name @ field` syntax may be used to rename what a field is called in subsequent
initializers.

When the field name is an underscore, it is treated like a field of type `()`.
Unlike initializers for named fields, there may be multiple such underscore
initializers in a single struct initializer. Underscore fields cannot be named
with `@`, but they may use `super let` to define variables that are accessible
in later initializers.

Note that the initializers for fields (but not underscores) are required to
implement `Init` in addition to `PinInit` unless the field is annotated with
`#[pin]` in the declaration of the struct.

If the initializer of any field fails, then the entire initializer fails.
Previously initialized fields (and super let variables) are dropped in reverse
declaration order, using the order they are declared in the `init` expression.

### Tuples and array syntax

If it matches `init (i1, ..., in)` or `init [i1, ..., in]`, then you get an
initializer whose `init` function runs the initializers in order to construct a
tuple or array. That is, for each `k` it calls `PinInit::init(ik, <ptr to kth
slot>)`.

There is no syntax for initializing tuples or arrays out of order, or for
accessing previously initialized values.

Tuples and arrays never require the initializers to implement `Init`. That is,
it behaves as-if the "fields" of the tuple or array are annotated with `#[pin]`.
This implies a decision that tuples and arrays always structurally pin their
contents, which Rust hasn't yet made a decision on, but structural pinning
is the natural default.

### Tuple structs

If it matches `init StructName(i1, ..., in)`, then you get an initializer whose
`init` function runs the initializers in order. That is, for each `k` it calls
`PinInit::init(ik, <ptr to kth field>)`.

Tuple structs are treated exactly the same as tuples, except that it follows the
same rules as structs with regards to `#[pin]` annotations on fields.

It's not possible to access previous fields using this syntax. It's also not
possible to reorder the fields. To do that, you may use the full struct syntax
instead:

```rust
init StructName {
    0: init_field_0(),
    2 @ foo: init_field_2(),
    1: init_field_1(&foo),
}
```

### Enums and unions

The syntax for structs and tuple structs also works with unions and enums. For
example, if you write `init MyEnum::MyCase { field: initer }`, then that will
initialize an enum. Same applies to `init Ok(initer)` to initialize a `Result`.

Note that enums are not syntactically different from the struct or tuple
struct cases, since `init MyEnum::MyCase` could syntactically just as well be
a struct inside a module.

The same applies to unions. In this case, the struct syntax should mention
exactly one of the union's fields.

### Arrays with repetition

If it matches `init [i; N]`, then the initializer evaluates `i` repeatedly `N`
times.

Note that repetition does not clone the initializer. Rather, it evaluates the
expression many times, similar to the body of a for loop.

### Blocks

If it matches `init { ... }` then the initializer just evaluates the block. The
last expression of the block must be another initializer.

This case may be used to define variables used by other parts of the
initializer. For example:

```rust
init {
    let value = expensive_logic();
    init [value; 1000]
}
```

creates an initializer that evaluates `expensive_logic()` once and copies the
output 1000 times, as opposed to `init [expensive_logic(); 1000]` that calls
`expensive_logic()` multiple times.

### When does `init` implement `Init`?

The opaque type of an `init` expression always implements `PinInit`, but only
implements `Init` if all initializers used to construct it implement `Init`.

### Capturing semantics

Any locals used inside an `init` expression are captured using move semantics
similar to `move || { ... }` closures.

## Pin annotations

Over an edition boundary, we introduce a new annotation `#[pin]` that may be
used on struct fields:

```rust
struct Foo {
    #[pin]
    pinned_field: F1,
    not_pinned_field: F2,
}
```

This annotation affects whether `init` expressions require initializers for the
fields to implement `Init`, but it also has several other effects:

### Destructors

When a field has a `#[pin]` annotation, you must use a different signature to
implement `Drop`.

```rust
impl Drop for Foo {
    fn drop(self: Pin<&mut Foo>) {
    }
}
```

### Unpin impl

The `Unpin` impl automatically generated by the compiler is modified so that it
only has where clauses for fields with the `#[pin]` annotation.

```rust
// compiler generated
impl Unpin for Foo
where
    F1: Unpin,
{}
```

If the `Unpin` trait is implemented manually, then it is an error to use the
`#[pin]` annotation on any fields of the struct.

Note that this requires that we change the default compiler generated
implementation of `Unpin`. This is possible over an edition boundary.

### Projections

Given a `Pin<&mut Struct>` to a `Struct` that is defined in the new edition and
doesn't have a manual `Unpin` implementation, you may project `Pin<&mut Struct>`
to either `Pin<&mut Field>` or `&mut Field` depending on whether the field is
annotated with `#[pin]` or not.

## Impl trait in dyn trait

Using `impl Trait` in return position no longer disqualifies a trait from being
dyn compatible. Specifically, the trait is dyn compatible if replacing `impl
Trait` with `dyn Trait` in the return type results in a valid unsized type.

Some examples:

```rust
struct Helper<T, U: ?Sized> {
    foo: T,
    bar: U,
}

// OK, return type is `dyn MyOtherTrait`.
trait Trait1 {
    fn foo(&self) -> impl MyOtherTrait;
}

// OK, return type is `Helper<String, dyn MyOtherTrait>`.
trait Trait2 {
    fn foo(&self) -> Helper<String, impl MyOtherTrait>;
}

// BAD, `Helper<dyn MyOtherTrait, String>` is an invalid type.
trait Trait3 {
    fn foo(&self) -> Helper<impl MyOtherTrait, String>;
}

// BAD, Default is not dyn compatible.
trait Trait4 {
    fn foo(&self) -> impl Default;
}

// BAD, `Box<dyn MyOtherTrait>` is not unsized.
//
// Not allowed because we probably want to just return `Box<dyn MyOtherTrait>`
// rather than `Init<Box<dyn MyOtherTrait>>`.
trait Trait5 {
    fn foo(&self) -> Box<impl MyOtherTrait>;
}
```

The compiler implements this by placing two entries in the vtable:

* The `Layout` for the concrete return type.
* A function pointer that takes a `*mut ()` and all arguments of the function,
  and writes the initialized value to the `*mut ()` pointer. It returns the
  metadata needed to construct a wide pointer to the value.

Based on this, it generates an unnameable type according to this logic:

```rust
// Given this trait
trait MyTrait {
    fn foo(&self, arg1: i32, arg2: &str) -> impl MyOtherTrait;
}

// The compiler generates this struct
struct FooInit<'a> {
    // the fields are just the arguments to `MyTrait`
    r#self: &'a dyn MyTrait,
    arg1: i32,
    arg2: &'a str,
}

impl PinInit<dyn MyOtherTrait> for FooInit<'_> {
    type Error = Infallible;

    fn layout(&self) -> Layout {
        self.r#self.vtable.layout
    }

    fn init(self, slot: *mut ()) -> Result<T::Metadata, Infallible> {
        self.r#self.vtable.foo_fn(
            slot,
            self.r#self as *const _,
            self.arg1,
            self.arg2
        )
    }
}
```

Whenever you call `<dyn MyTrait>::foo`, you receive a value of type `FooInit`.
The compiler generates one such type for each trait method using `impl Trait` in
return position. It always implements both `Init` and `PinInit`.

Trait objects using the above strategy for any method do not implement the
trait itself, since the method returns an initializer, and the initializer does
not implement the target trait. This is a deviation from the established
principle that `dyn Trait` implements `Trait.`

# Rationale and alternatives

## Only Init trait

It may be possible to start by only introducing an `Init` trait and then introduce a `PinInit` trait later. I believe that is probably doable in a forwards-compatible manner. This can avoid the complications with `#[pin]` annotations on struct fields.

# Prior art

## Rust for Linux

Rust for Linux has been using a crate called pin-init for a while that provides these features. The crate has a macro `pin_init!` that implements the `init` syntax. We have used it and it has worked well in real-world projects.

## Move constructors

Projects for performing C++ interop have developed several libraries that use very similar logic to the pin-init crate developed for the Linux kernel. For example, there is the moveit crate.

## Prototype

The pin-init crate does not come with support for returning `impl Trait` in `dyn Trait` methods, but there is a prototype of the crate with support for this. Please see it here:

<https://github.com/Rust-for-Linux/pin-init/tree/dev/experimental/dyn>
