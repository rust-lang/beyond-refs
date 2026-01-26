# Virtual places
 
{{#include ../cur.md}}

## Overview

This proposal centers around the concept of *places*. It offers a way to create
custom places, which do not need to be backed by actual memory--- hence the
name *virtual places*. These custom places can implement any of the existing
place operations and customize their behavior via operator traits. It also
extends the operation of borrowing a place to allow custom borrows.

To mark a type as a custom place, this proposal offers the `HasPlace` trait:

```rust
pub trait HasPlace {
    type Target: ?Sized;
}
```

Implementing this trait for a type `X` allows expressions of that type `x: X`
to be dereferenced: `*x`. This expression has the type `X::Target` and behaves
as a normal place expression, it can be read from, written to, and borrowed as
any place currently can in Rust. Whether any of these operations are allowed is
controlled by a trait and the borrow checker: the type `X` must implement the
place operation trait associated with the attempted operation. And the current
borrow checker state of the place must allow the operation to take place. For
example, if it is being used exclusively elsewhere, the borrow checker will
deny that operation.

Types that implement `HasPlace` are often smart pointers. Examples from the
standard library are `Box<T>`, `Arc<T>`, `Cow<'_, T>` and many more; all of
these examples implement `HasPlace<Target = T>`. In fact, all types that
implement `Deref` are examples for types that will implement `HasPlace` (we
also intend to make it a super-trait of `Deref`, but need to work on the
details). Non-`Deref` examples from the standard library are raw pointers and
`NonNull<T>`. Their place operations are unsafe, which this proposal explicitly
supports.

A simple example of what is possible with this proposal:

```rust
struct Struct {
    field: Field,
}

struct Field {
    accesses: usize,
}

impl Struct {
    unsafe fn reset(this: NonNull<Self>) {
        // We can borrow using `NonNull` directly:
        let field: NonNull<Field> = unsafe { @NonNull (*this).field };
        // Note that we can omit the dereference.
        // We can also use the canonical borrow, which fills `NonNull` for us:
        let field: NonNull<Field> = unsafe { @this.field };

        // We can also borrow using a totally different pointer if that is
        // supported by the underlying type:
        unsafe { std::ptr::write(@raw mut (*field).accesses, 0) };

        // Alternatively, we could also just have written to the field directly:
        unsafe { field.accesses = 0 };
        // Note that this drops the previous value, which does nothing for `usize`.
    }
}
```

## Place operations

Implementing `HasPlace for X` on its own is not particularly useful, as the
expression `*x` cannot be used anywhere without also implementing any place
operations. For example `*x = value;` requires implementing `PlaceWrite` for
`X`. The operations available on places are:

- [`PlaceBorrow`](./borrows.md) `&place`, `&mut place` and custom borrows
  `@Type place`,
- [`PlaceWrite`](./writes.md): `place = value;`,
- [`PlaceRead`](./reads.md): `let var = place;`, `match place { ... }`,
  `func(place)`, and many more.

Implementing these types requires `unsafe` and handling raw pointers, since we
want to support any kind of pointer expressible in Rust. Using them with the
usual place syntax is safe, since these traits are heavily integrated with the
borrow checker.

## Content overview

Giving Rust the capability to express custom places requires very invasive
changes. Since we want to make them compatible with many of Rusts other
features, we need to introduce a lot of concepts. Overall most of these are
generalizing already existing operations or capabilities. However, it is still
a lot to take in.

There are also several cycles in the dependency graph of the various concepts.
For this reason we sometimes introduce a concept in a simpler version only to
extend it later. Here is an overview of what the wiki currently covers:

- [Basic place expressions](./basic-place-expressions.md): place expressions as
  they currently exist in Rust. Later we will extend them to accommodate place
  wrappers.
- [Subplaces and projections](./place-projections.md): statically encoding the notion
  of a *subplace* that lives in the same allocation. Required for all place
  operations to support field access and indexing.
- [Place operations](./place-operations.md): A high level overview of place
  operations and their desugaring.
- [Place wrappers](./place-wrappers.md): how we support forwarding fields to
  support wrappers like `MaybeUninit<T>`.
- [Local and static places](./local-static-places.md): how we support borrowing
  local variables and statics.
- [Complete place expressions](./place-expressions.md): place expressions
  extended with place wrappers.
- [Dereferencing places](./dereferencing-places.md): how nested dereferences
  are desugared in the other operations.
- [Writing to places](./writes.md): how writes are desugared and implemented
  via `PlaceWrite`.
- [Reading from places](./reads.md): how reads are desugared and implemented
  via `PlaceRead`.
- [Borrowing places](./borrows.md): how borrows are desugared and implemented
  via `PlaceBorrow`.
- [Dropping places](./dropping.md): how to drop places and partially moved out
  smart pointers.
- [Borrow checker integration](./borrow-checker.md): how we integrate the
  various operations into the borrow checker and allow customizing its
  behavior.
- [Canonical borrow](./canonical-borrow.md): syntactic sugar for borrowing
  places without changing the pointer type.
- [Autoref and method resolution](./autoref.md): how we can leverage custom
  borrows to call methods with custom smart pointers.
- [Dynamically sized types and metadata](./metadata.md): how we handle pointers
  to DSTs.

It is recommended to read through these sections in order, as we start out with
a simpler concept of place expressions, which is required to make sense of
place wrappers, which are in turn needed by place expressions. At the end of
this section, we are going to describe the missing parts and open questions.

## Motivation

Overall, with this proposal, we intend to make all currently compiler built-in
types expressible in normal Rust code. This permits libraries to construct
types that *feel* like they are built-in, but in reality are implemented
entirely by the library. It also has the advantage of making the built-in types
no longer special, which makes them easier to understand.

### Motivating examples

The built-in types that we aim to directly support with this proposal are:

- `&T` and `&mut T`, they:
  - have direct borrow checker support,
  - support [reborrowing](../reborrow/index.md),
  - have ergonomic syntax to access fields and create them pointing to local
    variables.
- `*const T` and `*mut T`, they:
  - can be dereferenced and are fully integrated with the place expressions of
    Rust,
  - require `unsafe` to use them in any way.
- `Pin<P>` (not special at the moment, but could be as part of
  [pin ergonomics](https://github.com/rust-lang/rust/issues/130494)),
- `Box<T>`, it:
  - allows partially moving out the struct like local variables.

The types that will end up feeling built-in, but could be implemented by a user
Rust library:

- `NonNull<T>`,
- `cell::Ref<'_, T>` and `cell::RefMut<'_, T>`,
- `MaybeUninit<T>`, `Cell<T>`, and `UnsafeCell<T>`.

More examples can be found in the [design
meeting](https://hackmd.io/@rust-lang-team/S1I1aEc_lx) on the motivations for
field projections.

## Running example

Throughout this part of the wiki, we will use a simple custom `Box` as a
running example to highlight how one would use the various concepts we're
introducing.

Here is our definition:

```rust
pub struct MyBox<T: ?Sized>(NonNull<T>);
```

We'll start with this and nothing else. At the end of this chapter, we will
have a fully functional custom box that behaves almost exactly like `Box`.

As our first impl, we give the `HasPlace` trait:

```rust
impl<T: ?Sized> HasPlace for MyBox<T> {
    type Target = T;
}
```

Now writing `*b` for `b: MyBox<T>` will be possible. However, using this
expression anywhere will lead to a compiler error, as we have not yet
implemented any of the place operation traits. We'll introduce them one-by one
after the preliminaries are out of the way.

Note that we have explicitly not implemented `Drop` at this point. That will
come in the section on [dropping places](./dropping.md).

## Open questions

- How to make `Deref` a subtrait of `HasPlace` in a backwards-compatible way?
- Is there a place operation that should replace `std::ops::Index[Mut]`?
- How are `repr(packed)` structs supported by projections?

## Resources

- [Truly First-Class Custom Smart Pointers | Nadri’s musings](https://nadrieril.github.io/blog/2025/11/11/truly-first-class-custom-smart-pointers.html) Nov 2025, origin of the idea of virtual places
- [Design meeting: Field projections | t-lang design meeting](https://hackmd.io/@rust-lang-team/S1I1aEc_lx) Aug 2025, motivation for a field projections feature
