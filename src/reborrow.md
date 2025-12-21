# Reborrow

Reborrowing is an action performed on exclusive references which creates a copy
of the source reference and marks it disabled for reads and writes, or writes
for the duration of the copy's existence. This means that the exclusivity of the
reference is retained despite a copy being made, as only one copy can be used at
a time.

The true power of reborrowing is today only available to Rust's exclusive `&mut`
references. Going beyond references means enabling its power for user defined
exclusive reference types as well.

{{#include stub.md}}

## Use cases

The basic use-case of reborrowing is the following: we shall assume some custom
type that has exclusive reference semantics, meaning that only one copy of it is
active at a given time, and we want the compiler to automatically inject
reborrowing operations into our code as we write it.

Example:

```rust
#[derive(Reborrow)]
struct CustomMut<'a, T>(*mut T, PhantomData<&'a mut ()>);

fn f(a: CustomMut<'_, u32>) {
    inner(a);
    inner(a);
}
```

In current Rust, this code would not compile due to `CustomMut: !Copy`. With
reborrowing enabled to custom types, reborrowing operations will be
automatically injected at each use site of `a`, making the code compile.

Exclusive reference semantics are most useful when paired with shared
references. Shared references are simply `Copy` types, so their semantics are
not particularly interesting on their own. The interesting part is how exclusive
references can be coerced into shared references. In the above example, it isn't
actually even clear if the two `inner` method calls take a `CustomMut<u32>`, or
if they perhaps take some `CustomRef<u32>` instead. Whichever it is, the code
should work the same.

### Wrapper types

Currently Rust does not automatically reborrow `Option<&mut T>` or similar
wrapper types. Conceptually, there is no reason why `&mut T` should be
reborrowable but `Option<&mut T>` would not. The only difference between the two
types is that one can also be null.

With Reborrow traits, reborrowing wrapper types of exclusive references becomes
possible using blanket implementations.

```rust
impl<T: Reborrow> for Option<T> { /* ... */ }
```

### Custom semantics and markers

Custom types with exclusive reference semantics are unlikely to be 100%
equivalent to Rust's exclusive references: if they were, then there'd hardly be
any need for custom types. A `CustomMut` type might have exclusive lifetime
semantics while not guaranteeing exclusive access to the pointed-to data, in
which case the exclusivity is there to protect against accidental misuse but
does not give the compiler any optimisation guarantees. This sort of a
shared-mutable reference is for instance useful when interacting with C++. Some
custom types may contain what is effectively a Rust reference combined with
extra metadata to enable eg. referencing a subset of a matrix.

It is also possible for custom types with exclusive reference semantics to not
contain any actual pointers at all. This is useful in encoding exclusive access
to data in the function's indirect context. As an example, embedded systems
sometimes use reborrowable ZST marker types to pass exclusive access to hardware
peripherals through their call stacks.

```rust
struct CustomMarker<'a>(PhantomData<&'a mut ()>);
```

### Exclusive reference collections

In some cases it can be useful to group up multiple exclusive references into a
single collection.

```rust
struct MutCollection<'a, 'b, 'c> {
    a: &'a mut A,
    b: &'b mut B,
    c: &'c mut C,
}
```

Reborrowing such a collection as exclusive means simply reborrowing each
exclusive reference individually. Coercing the collection into shared is more
complicated: for a collection of three exclusive references there are 7
collections of three references with differing exclusivity that this can be
coerced into. If including collections with two members, the number goes up
again.


## Approaches

The current approach to reborrowing in user-land is based on reborrowing
functions. The current work in the Reborrow traits lang experiment is based
on a function-less implementation.

### Function-based reborrowing

A form of reborrowing can today be implemented in user-land using a pair of
reborrowing functions:

```rust
fn reborrow<T>(t: &mut T) -> T;
fn coerce_shared<U, T>(t: &T) -> U;
```

This captures the most important features of reborrowing: a source `T` has
exclusive or shared access asserted on it, and a new `T` (or `U`) is produced
from it. Usually these functions are defined either directly on a type, or a
specific `Reborrow` trait is created, possibly with a separate `CoerceShared`
trait. However, this approach comes with downsides related to the lifetimes of
the references that the functions take and the existence of user-controlled
functions in the first place. These downsides are explained in more depth below.

### Function-less reborrowing

The current Reborrow traits lang experiment aims for a function-less
implementation of reborrowing. For exclusive reference semantics this is mostly
trivial: the concrete code generated for an exclusive reborrow is equivalent to
that of `Copy`, and only additional lifetime semantics must be considered on
top.

This gives us a trivial, derivable `Reborrow` trait:

```rust
trait Reborrow {}

#[derive(Reborrow)]
struct CustomMarker<'a>(PhantomData<&'a mut ()>);
```

There are some limitations that we wish to impose on types that derive (or
implement) this trait.

1. The type must not be `Clone` or `Copy`. (Alternatively, blanket-implement
   `Reborrow for T: Copy`, but `Clone + !Copy` types cannot be `Reborrow`.)
1. The type must not have a `Drop` implementation.
1. The type must have at least one lifetime. (If `Copy` types blanket-implement
   `Reborrow` then this limitation cannot be placed, but manually deriving
   `Reborrow` on a lifetime-less type should be an error.)
1. The result of a `Reborrow` operation is simply the type itself, including the
   same lifetime.

Coercing into a shared reference is much more complicated: for custom types, the
shared reference type must be a different type than the exclusive reference
type. That brings up various questions, such as the number of fields that each
custom type has and the order in which they are laid out.

Still, these are only problems for the compiler and not really the user. From a
normal user's perspective, coercing an exclusive reference into shared is simple
enough: just define the source and target types and their relationship.

This gives us the following trait definition:

```rust
trait CoerceShared<Target: Copy>: Reborrow {}

struct CustomMarkerRef<'a>(PhantomData<&'a ()>);

impl<'a> CoerceShared<CustomMarkerRef<'a>> for CustomMarker<'a> {}
```

Again some limitations appear, this time mostly expressed on the trait directly.

1. The type implementing `CoerceShared` must also implement `Reborrow`.
2. The result of the `CoerceShared` operation must be a `Copy` type.
3. The lifetime of the result must be equivalent to the source.
4. The target type must be relatable to the source type field-by-field.

As long as we deal with types that only have a single lifetime, this is all
quite simple and straightforward.

### `Reborrow` and `CoerceShared` with collections

To be fleshed out...

Main points: how to find the "base cases" where `Reborrow` or `CoerceShared`
performed on a collection terminates? That these operations are recursive seems
fairly obvious, but how that recursion is terminated is unclear. `&mut T` and
`&T` are of course base cases that terminate the recursion, but what about
`CustomMarker`?

Do we recurse inside of it and find the lone `PhantomData`? If we do, then how
do we decide to perform exclusive reborrowing as opposed to simply making a copy
of the `PhantomData` to produce a brand new, unconnected `CustomMarker`? If we
do not recurse inside of `CustomMarker`, why not? It cannot be only because it
is `Reborrow`, as that would forbid collections from containing collections. It
could be because it only has one lifetime and is thus trivial to reborrow, but
what if it did have two...?

```rust
#[derive(Reborrow)]
struct CustomMarkerOfTwo<'a, 'b>(PhantomData<&'a mut ()>, PhantomData<&'b ()>);
```

Did the user intend for both of these lifetimes to be used as exclusive, or did
they perhaps intend only one of them to be so? Do we need a new
`PhantomExclusive<'a>` marker type to act as a base case here? What is the
variance of `'a` in `PhantomExclusive<'a>`?

## Downsides of function-based reborrowing

#### Lifetime shortening in the function formalisation

This function formalisation is not a functional solution for true reborrowing.
The problem is that any outer function that calls these functions generally
needs to create the `&mut T` or `&T` for the calls locally on the stack, and
thus the resulting `T` or `U` captures the local stack lifetime. This means that
the result cannot be returned from the outer function.

Example:

```rust
fn outer_fn<'a>(t: CustomMut<'a, u32>) -> &'a u32 {
    // does not compile: returns a value referencing data owned by the current
    // function
    inner_fn(t.reborrow())
}
```

Compare this to Rust references: the compiler understands that the result of
reborrowing a `&mut` produces a new reference that can be extended to the
original reference's lifetime.

```rust
fn outer_fn<'a>(t: &'a mut u32) -> &'a u32 {
    // compiles despite explicit reborrowing
    inner_fn(&mut *t)
}
```

Although, it is possible to make it not compile by explicitly performing a
borrow of the reference:

```rust
fn outer_fn<'a>(mut t: &'a mut u32) -> &'a u32 {
    // no longer compiles: returns a value referencing data owned by the current
    // function
    inner_fn(&mut t)
}
```

The function formalisation means that calling `fn reborrow` performs a borrow of
the reference, making the code not compile. This can be fixed by simply not
calling the `reborrow` function in the example code, but that may not be a valid
solution in the long term. For one, if the reborrowing function calls become
automatically injectable by the compiler, it may not be feasible to perform the
necessary analysis to eliminate these unnecessary and problematic `reborrow`
calls.

Furthermore, in a post-Polonius borrow checker world it will become possible for
code like this to compile:

```rust
fn inner_fn<'a>(t: &'a mut u32) -> Result<&'a u32, &'a u32> {
    // ...
}

fn outer_fn<'a>(t: &'a mut u32) -> Result<&'a u32, &'a u32> {
    let result = inner_fn(t)?;
    if result > 100 {
        Ok(t)
    } else {
        Err(t)
    }
}
```

The example code is of course meaningless, but the point is that a reborrow of
`t` must always happen, yet the lifetime expansion of the result of `inner_fn`
depends on whether the function returns `Ok` or `Err`. In the `Err` branch the
result's lifetime must expand to that of the source `t`, but in the `Ok` branch
it must shrink to re-enable usage of the source `t`.

The function-based approach to Reborrow traits will simply not work here, not
unless the compiler blindly trusts that the reborrowing functions do no evil and
performs the same sort of conditional lifetime expansion on the result. Note
that this is unsound in the general case.

#### User-controlled code

The other big downside tha function-based approach is that reborrowing is fully
invisible in the source code, and having user-controlled code appear where there
is none visible is not very Rust-like.

Consider eg. the following code:

```rust
struct Bad;

impl Reborrow for Bad {
    fn reborrow(&mut self) -> Self {
        println!("I'm bad!");
        Self
    }
}

fn main() {
    let bad = Bad;
    let bad = bad;
}
```

Depending on the exact implementation choices, this might print out the "I'm
bad" message. This is especially problematic if the compiler chooses to blindly
trust the reborrow methods and always assume their resulting lifetime can be
extended.

```rust
struct Unsound<'a>(&'a mut &'a u32, u32);

impl<'a> Reborrow<'a> for Unsound<'a> {
    fn reborrow(&'a mut self) -> Self {
        let data = &self.1;
        self.0 = data;
        Self(self.0, self.1)
    }
}
```

This would absolutely not compile today, but if the compiler did truly believe
that reborrow methods can do no wrong then this might just pass through the
compiler's intentional blindspot and become a soundness hole.

## Resources

[Tracking Issue for Reborrow trait lang experiment · Issue #145612 · rust-lang/rust](https://github.com/rust-lang/rust/issues/145612)

[Reborrow traits - Rust Project Goals](https://rust-lang.github.io/rust-project-goals/2025h2/autoreborrow-traits.html), Jul 2025

[rfcs/text/0000-autoreborrow-traits.md at autoreborrow-traits · aapoalas/rfcs](https://github.com/aapoalas/rfcs/blob/autoreborrow-traits/text/0000-autoreborrow-traits.md), May 2025

[Abusing reborrowing for fun, profit, and a safepoint garbage collector](https://github.com/aapoalas/abusing-reborrowing/tree/main) (conference talk with examples), Feb 2025
