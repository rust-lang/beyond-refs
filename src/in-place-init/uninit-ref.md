# `&uninit` reference and `Initialised` proof marker

Out pointers done manually are verbose and quite unergonomic. Making out
pointers a first class citizen aims to improve this situation. The first step is
replacing `Uninit<'a, T>` with a custom `&uninit T` reference type.

The `&uninit T` is a mutable, reborrowable reference type that is guaranteed to be fully
uninitialised at the start of each function and that the compiler can therefore
consider as being equivalent to an uninitialised local place:

```rust
fn foo(x: &uininit X) {
    let y: X;
    // x and y are equivalent, uninitialised places. Only their lifetimes differ.
}
```

The borrow checker can therefore track the initialisation status of `&uninit T`
the same way it tracks the deinitialisation status of partially moved-out-of
structs today, and how it is likely to track the initialisation status of
partially initialised structs soon. When an `&uninit T` has been fully
initialised, it can be turned into an `Initialised` proof marker which is a
zero-sized type that just carries the lifetime `&uninit T` did, again as
invariant, but does not carry the type `T`.

The `&'a uninit T` is invariant on its lifetime `'a` as it was with the
`Uninit<'a, T>` formulation, but the reborrowability is new: this is done to
enable functions to skip returning the `&uninit T`'s pointer after initialising
it. Consider a basic initialisation case:

```rust
struct X(u32, u32);

fn init_x(x: &uninit X) -> Initialised {
    x.0 = 3;
    x.1 = 4;
    x
}
```

In the `Uninit<'a, T>` formulation this function would return `InPlace<'a, T>`
which, as in eg. C++, means that the actual ABI of this function would be to
take a pointer `*mut X` and return the same pointer `*mut X` after
initialisation. Returning the pointer is meaningful in the sense that it acts as
proof that its contents was initialised, but in Rust we already have proof of
initialisation in the invariant lifetime: we do not need the pointer to be
returned explicitly (it will likely instead be stored in a callee saved
register).

Whether to return the pointer or not becomes especially meaningful when fallible
initialisation is considered:

```rust
fn try_init_x(x: &uninit X) -> Result<Initialised, E> {
    x.0 = try_get()?;
    x.1 = try_get()?;
    x
}
```

A `Result<Initialised, ()>` fits into a single register if `Initialised`
contains the non-null initialised pointer, but if a non-empty error variant is
returned then this return value requires at least two registers or a stack
spill. To avoid this, we make `&uinit T` reborrowable and `Initialised` a ZST.

Basic usage of `try_init_x` then looks like this:

```rust
let x: X;
x <- try_init_x(&uninit x)?;
```

The `place <- Initialised` syntax (which would optionally just be `place =
Initialised` with extra compiler magic) acts as a notarisation/proof of `x`
having been successfully fully initialised inside of `try_init_x` and that after
this point this function holds a valid `X` with full drop responsibility of it.

Using `try_init_x` inside a function that itself takes `&uninit X` looks like this:

```rust
fn init_x(x: &uninit X) -> Initialised {
    // Note: x is reborrowed in `try_init_x(x <== here)`, hence why we can still
    // notarise `x` after the call.
    *x <- try_init_x(x).unwrap();
    x
}
```

The `Initialised` proof we get from `try_init_x` is "used up" to notarise the
local state of `x: &uninit X`, proving that the `X` has been fully initialised.
After this point this function holds a valid `X` with full drop responsibility
of it, but returning `x` from the function coerces it into an `Initialised`
proof without dropping the `X`, thus giving the drop responsibility to the
caller.

## Special powers of `Initialised`

The `Initialised` type is, fundamentally, a normal ZST marker type carrying only
an invariant lifetime but it has two special powers:

1. It can be used to notarise uninitialised places (give proof of
   initialisation).

2. It cannot be dropped.

3. It can only be created from a fully initialised `&uninit T` which then
   consumes said `&uninit T` instead of reborrowing it.

The first power we have already seen above: this is compiler internal magic that
works to affect the type state of an uninitialised place the same way that
assigning a value to an uninitialised place does.

The second power is something that exists even in normal Rust code today but
that is rarely very useful: effectively we could say that `Initialised`
implements `Drop` in the following way:

```rust
impl Drop for Initialised<'_> {
    fn drop(&mut self) {
        const { panic!("Initialised cannot be dropped – notarise the &uninit immediately instead") };
    }
}
```

This is a limitation that we place on the usage of `Initialised` in order to
avoid the need for `Initialised` to carry the pointer: dropping `Initialised`
would need to perform `drop_in_place` on the now-initialised `&uninit` memory
but because it does not carry the pointer data, it cannot perform said drop.
Hence, `Initialised` must be undroppable.

This could be a nasty limitation, but luckily in our case it just means that
notarisation must be done immediately after receiving an `Initialised`,
consuming it, after which point the `&uninit T` takes up the drop responsibility
and no more nasty business happens.

```rust
// This is erroneous:
let i: Initialised = try_init_x(x)?;
maybe_panic(); // ERROR: potentially dropping `i`
x <- i;

// But it can be fixed trivially:
let i: Initialised = try_init_x(x)?;
x <- i;
maybe_panic();
```

The third power, converting a fully initialised `&uninit T` into an
`Initialised` as the only means of creating an `Initialised`, is the backbone of
the proof system. This has to tie in with the compiler's field initialisation
tracking both on the output end (creating a proof from known-initialised
`&uninit T`) and at the input end (proving an `&uninit T` or its field
initialised through notarisation with an `Initialised`).

## Composition

Equivalently to struct field-by-field initialisation, field-by-field `&uninit`
references and notarisation must be possible.

```rust
fn init_struct(s: &uninit Struct) -> Result<Initialised, E> {
    s.base <- init_base(&uninit s.base)?;
    // Note that &s.base and &mut s.base are now legal since we know s.base is initialised.
    s.self_ref <- init_base_self_ref(&uninit s.self_ref, &s.base);
    Ok(s)
}
```

## API sketch

* [Rust playground link](https://play.rust-lang.org/?version=nightly&mode=debug&edition=2024&gist=71fc6309242bcb601ec150d7461413c1)
