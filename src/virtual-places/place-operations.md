# Place operations

We still require some preliminaries to describe all of the *place operations*.
However, these preliminaries benefit a lot from knowing how the place
operations are designed. For this reason we give a general overview of how they
are conceptualized here. In addition we cover two important concepts used in
their desugarings later. The main operations are reading `PlaceRead`, writing
`PlaceWrite`, and borrowing `PlaceBorrow`. Auxiliary operations are
dereferencing `PlaceDeref` and dropping `PlaceDrop` & `DropHusk`.

Place operations operate on [*place
expressions*](./basic-place-expressions.md). The main operations have special
syntax: `place = value;`, `let val = place;` or `@Type place`. The auxiliary
operations of dereferencing and dropping don't have direct syntax
representation, they are inserted by the desugaring algorithms and the borrow
checker.

A place operation has (as any operator in Rust) a trait associated with it.
They all follow a similar pattern:
- It is an `unsafe` trait, because it tightly integrates with the borrow
  checker and implementing it in the wrong way can result in undefined
  behavior.
- It has `HasPlace` as a supertrait. This is so we can talk about
  `Self::Target` within the trait.
- It has a generic parameter `P: Projection<Source = Self::Target>`, which
  represent the subplace the operation is performed on.
- It declares an `unsafe fn` for the operation, which takes a raw pointer to
  `Self` as the argument to apply the operation on. This is because several
  operations might be happening in parallel on the same object subject to
  borrow checker restrictions.

Putting all of this together we obtain the following general operation:

```rust
pub unsafe trait PlaceOp<P>: HasPlace
where
    P: Projection<Source = Self::Target>,
{
    unsafe fn perform(this: *const Self, proj: P, args: ...) -> ...;
}
```

Keep in mind that operations might contain more generics & parameters depending
on their exact use-case. Their function always takes a raw pointer to `Self`
and the projection if there is one.

## Dereferencing

The main operations implicitly use the dereference operation when more than one
dereference occurs in the place expression they are applied to. One very
important principle to keep in mind with this is the following:

> The outermost dereference in the place expression **is part** of the place
> operation itself. All *inner* dereferences correspond to the place
> dereference operation.

This is best illustrated with some examples. We will use the write operation
and the familiar `Box` type for this purpose, since we have not fully defined
them and will only do so later. In the first example, there only is a write
operation, despite us using an explicit dereference:

```rust
let mut b = Box::new(42);
*b = 24;
// will be desugared to:
PlaceWrite::write(&raw const b, 24);
// note the signature of the write operation: `fn(*const Box<i32>, i32)`
```

When dealing with a nested box, we have to use two dereferences, which will
turn the innermost dereference into the place dereference operation. The outer
dereference will again be associated with the write operation and "vanish" for
that reason:

```rust
let mut b = Box::new(Box::new(42));
**b = 24;
// will be desugared to:
PlaceWrite::write(PlaceDeref::deref(&raw const b), 24)
// note the signature this deref: *const Box<Box<i32>> -> *const Box<i32>
```

For this reason we will introduce the desugaring of dereferences first after
the other preliminaries and then move on to the main operations.

## Local variables and statics

Local variables and static variables can be involved in place operations
directly without dereferences. For example directly assigning a value to a
local variable, or borrowing a static:

```rust
let mut b = Box::new(42);
b = Box::new(24); // stores 24 in a *new* box.

static GREETING: &'static str = "Hello!";
let greeting: &'static &'static str = &GREETING;
```

One way to support writing and reading would be by hardcoding it in the
compiler. But for borrowing that cannot be done, since we want to allow custom
borrows: `@MyPtr place` should be of type `MyPtr<...>`. The central question
is:

> How should a user specify that their smart pointer can be created pointing to
> a local or a static?

The answer is of course *compiler magic*. It takes the form of two lang items
(which are used to drive custom behavior in the compiler for types):

```rust
#[lang = "static_place"]
pub struct StaticPlace<T>(T);

#[lang = "static_place"]
pub struct LocalPlace<T>(T);
```

When borrowing a local place of type `T`:

```rust
let t: T = ...;

let x = @MyPtr t;
```

The compiler will supply the `PlaceBorrow` operation not with a `*const T`, but
a `*const LocalPlace<T>` instead. Additionally, the type of which we invoke the
operation is `LocalPlace<T>`. With some further compiler magic, we will allow
implementing `PlaceBorrow` for `LocalPlace` such that any custom pointer can
borrow locals if their author adds the correct impl. The same applies to
`StaticPlace`.

Since we already require this workaround for borrows, we can simplify the
desugaring algorithms by also using `LocalPlace` and `StaticPlace` for reads
and writes. The actual compiler implementation might not do this for
performance reasons.

One important complication that we have not covered is that this does not
compose with taking a subplace of `T`:

```rust
let t: T = ...;

let x = @MyPtr t.proj;
```

However, we also have a solution for that: `PlaceWrapper`, which we cover next.
