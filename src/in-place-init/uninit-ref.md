# `&uninit` references

Introduce a first-class `&uninit T` pointer type whose initialisation state is
tracked by the compiler. Make it possible to return a marker proving
full-initialisation status of the `&uninit T` from functions.

## Semantics

An `&uninit T` is a data pointer pointing to uninitialised, partially
initialised, or fully initialised data. The state of the initialisation is not
queryable using runtime functions; it is tracked at compile-time only.

The `&uninit T` is reborrowable into another `&uninit T`. The lifetime `'a` in
`&'a uninit T` is invariant and guaranteed unique. An initialised field of an
`&uninit T` can be reborrowed as `&(mut) Field`.

When initially received, be it by creation or as a parameter, the `&uninit T` is
fully uninitialised. When dropped or reborrowed, the `&uninit T` drops the `T`
in place if initialised, or drops all initialised fields of `T` in place
otherwise.

An `&uninit T` can be split into multiple `&uninit Field` references in the same
way as a `&mut T` can be split into `&mut Field` references.

An `&uninit T` can be initialised by writing into it, or by using an
initialisation proof marker `Initialised<'_>`. The lifetime of `'a` of
`Initialised<'a>` is invariant. A field-wise or fully initialised `&uninit T`
can be reborrowed into an `Initialised<'_>`, which resets the `&uninit T`'s
initialisation status to fully uninitialised without performing `Drop` in place.

## Initialising `&uninit T` or a local uninitialised `T`

### One-shot initialisation

An `&uninit T` can be fully initialised by writing to it.

```rust
let r: &uninit T = ?;
*r = T::new();
```

This marks the `&uninit T` as fully initialised and arms its `Drop` method.

Equivalently for a local uninitialised `T`:

```rust
let r: T;
r = T::new();
```

### Field-wise initialisation

An `&uninit T` where `T` is a struct can be initialised field-by-field using a
special initialisation syntax:

```rust
let r: &uninit T = ?;
r.field1 <- Field1::new();
r.field2 <- Field2::new();
r <- _;
```

Alternatively, reuse normal assignment syntax:

```rust
let r: &uninit T = ?;
r.field1 = Field1::new();
r.field2 = Field2::new();
*r = _;
```

Initialising a field makes the borrow checker track that field's value as an
individual value. Initialising all fields still tracks the fields as individual
values, ie. it does not consider the `&uninit T` to yet contain an initialised
`T` and does not therefore arm the `Drop` method of `T`.

The final `r <- _;` or `*r = _;` line is therefore required to complete the
initialisation; this finally arms the `Drop` method of `T`.

Equivalently for a local uninitialised `T`:

```rust
let r: T;
r.field1 = Field1::new();
r.field2 = Field2::new();
r = _;
```

### Initialisation functions

An `&uninit T` can be passed to a function as a parameter: the callee will
consider the `&uninit T` to be fully uninitialised. The callee can signal to the
caller that it has fully initialised the `&uninit T` by returning an
initialisation proof, here called `Initialised<'_>`.

```rust
let r: &uninit T = ?;
let proof: Initialised<'_> = init_t(r);
r <- proof;
```

The initialisation proof is "notarised" onto the `&uninit T` using the `r <-
proof;` syntax. An alternative is to use the standard pointer write syntax:

```rust
let r: &uninit T = ?;
*r = init_t(r);
```

This requires special handling in the compiler as `Initialised<'_>` is not equal to `T`.
Another alternative would be to make dropping the proof automatically notarise the `&uninit T`:

```rust
let r: &uninit T = ?;
init_t(r);
```

This requires special handling in the compiler as dropping of `Initialised<'_>`
would have to happen immediately on the second line above, and its `Drop`
implementation would have to find the exact `r: &uninit T` based on the
invariant and guaranteed unique lifetime that they share, and notarise it.

A local uninitialised `T` cannot be initialised using an initialisation function
without taking an `&uninit T` reference to it:

```rust
let r: T;
init_t(&uninit t);
```

## Syntax sugar

Having to write out `&uninit` is a nuisance in most case. Many in-place
initialisation cases are dead-simple. For these cases, it would make sense to
have simple syntax sugar to deal with the nuisance.

One possibility would be to use the magic `_` binding on the right-hand side of
an assignment with the meaning of "references the left-hand side"; this
reference would necessarily be a an `&uninit T` since the left-hand side must be
uninitialised when the right-hand side is being evaluated.

This makes calling constructor functions much more pleasant:

```rust
fn init_r(r: &uninit Struct) -> Initialised<'_> { ... }

let r = init_r(_);
```

This would also work with fields:

```rust
struct Struct {
    field1: Field1,
    field2: Field2,
}

let r: Struct;
r.field1 = init_field1(_); // &uninit Field1 -> Initialised
r.field2 = init_field2(_); // &uninit Field2 -> initialised
r = _;
```

This also applies to `r = _;` which now desugars into `r = &uninit r`: if all of
`r`'s fields are fully initialised then `&uninit r` can be reborrowed into an
`Initialised<'_>`. The initialisation proof can then be assigned into `r` to
finish its initialisation.

## Pros & cons

### Pros

1. Out pointers are the way that in-place initialisation actually works on the
   concrete, on-the-metal level. You cannot make the problem simpler than it
   actually is.

2. Out pointers are explicit and flexible: initialising functions (constructors)
   are free to choose their calling convention, and functions taking multiple
   out pointers are not an issue.

3. Initialisation proofs enable making `&uninit T` reborrowable: an alternative
   approach of returning `&init T` pointers requires `&uninit T` to be a
   `Move`-only type. This also enables very efficient initialisation function
   APIs.

### Cons

1. `&uninit T` and `Initialised<'_>` are often explicitly spelled out; they are
   more verbose than automatic solutions. Syntax sugar helps a lot though.

2. The implementation requires a non-trivial amount of new compiler features.

3. `&uninit T` does not itself provide a direct path to solving eg. in-place
   `Box` or `Rc` initialisation. The most direct solution of adding new APIs
   that take an `impl FnOnce(&'a uninit T) -> Initialised<'a>` run into the same
   issues as `impl Init` does: return type of the `FnOnce` spills into the
   return type of the new APIs, and the new APIs will need various variants to
   match status quo, leading to API bloat.

## Examples

### Correct usage

These are examples of correct, unproblematic usage.

#### One-shot initialisation

```rust
struct Struct {
    field1: Box<u32>,
    field2: Box<u32>,
}

fn init_s(s: &uninit Struct) -> Initialised<'_> {
    *s = Struct::default();
    s
}

let s = init_s(_);
```

#### Field-wise initialisation

```rust
struct Struct {
    field1: Box<u32>,
    field2: Box<u32>,
}

fn init_b(s: &uninit Box<u32>) -> Initialised<'_> {
    *s = Default::default();
}

let s: Struct;
s.field1 = init_b(_);
s.field2 = init_b(_);
s = _;
```

#### C++ constructor ABI

A C++ constructor takes an `*mut self` parameter and returns it as (hopefully)
initialised. Implementing the equivalent with `&uninit T` is possible, though it
requires either that `Initialised<'_>` can be wrapped in `ManuallyDrop` (so it
must not be a true linear type), or that the planned `unsafe fn drop_in_place`
overriding feature is used.

```rust
struct Class { ... };

#[repr(transparent)]
struct Init<'a, T>(&'a uninit T, Initialised<'a>);

impl Destruct for Init<'_, T> {
    unsafe fn drop_in_place(&mut self) {
        // NOTE: self.0 is considered fully uninitialised here.
        // Mark self.0 as fully initialised: how to move out of self.1 though?
        self.0 = self.1;
        // Here self.0 is exiting the function and gets dropped in place.
    }
}

extern "C" fn lib__Class__new(c: &mut uninit Class) -> Init<'_, Class> {
    c.field1 = init_field1(_);
    // ... other field inits here ...
    // notarise field-wise initialised &uninit Class, arming its `Drop` and then
    // immediately moving the Drop responsibility into proof.
    let proof: Initialised = c;
    // NOTE: c is now considered fully uninitialised as proof carries Drop
    // responsibility. Class is not uninitialised here.
    Init(c, proof)
}
```

#### Calling a C++ constructor

```rust
struct Class { ... };

#[repr(transparent)]
struct Init<'a, T>(&'a uninit T, Initialised<'a>);

impl Destruct for Init<'_, T> { ... } // same as above

impl<'a, T> Init<'a, T> {
    fn into_proof(self) -> Initialised<'a> {
        // NOTE: self.0 is considered fully uninitialised here.
        self.1
        // NOTE: self.0 is still considered fully uninitialised here and thus no
        // drop in place is performed.
    }
}

#[link(name = "lib")]
unsafe extern "C" {
    fn lib__Class__new<'a>(&'a uninit Class) -> Init<'a, Class>;
}

let c: Class = unsafe { lib__Class__new(_) }.into_proof();
```

#### Fallible initialisation

```rust
fn try_init_s(s: &uninit Struct) -> Result<Initialised<'_>, dyn Error> { ... }

let s = try_init_s(_)?;
```

### Incorrect usage examples

These are examples of incorrect usage that do not compile.

#### Reference to partially initialised struct

```rust
struct Struct {
    field1: Box<u32>,
    field2: Box<u32>,
}

fn init_b(s: &uninit Box<u32>) -> Initialised<'_> {
    *s = Default::default();
}

let s: Struct;
s.field1 = init_b(_);
let: &Struct = &s; // ERROR: used binding `s` isn't initialized
s.field2 = init_b(_);
let: &Struct = &s; // ERROR: used binding `s` isn't initialized
s = _;
let: &Struct = &s; // OK
```

### Misuse examples

These are examples of correct but problematic usage: they compile but contain
mistakes.

#### Partial initialisation undone

```rust
struct Struct {
    field1: Box<u32>,
    field2: Box<u32>,
}

fn init_s(s: &uninit Struct) -> Initialised<'_> {
    // NOTE: s is considered fully uninitialised at function entry here.
    *s = Struct::default();
    s
}

let s: Struct;
s.field1 = Default::default();
// MISTAKE: field1 was initialised and is dropped here.
s = init_s(_);
```

#### Partial initialisation undone by return

```rust
struct Struct {
    field1: Box<u32>,
    field2: Box<u32>,
}

fn half_init_s(s: &uninit Struct) {
    s.field1 = Default::default();
    // MISTAKE: there is no way to return the half-initialised state. Thus,
    // field1 is dropped here.
}

let s: Struct;
half_init_s(&uninit s);
s = Default::default();
```

#### Field-wise initialisation unfinished

```rust
struct Struct {
    field1: Box<u32>,
    field2: Box<u32>,
}

impl Drop for Struct {
    fn drop(&mut self) {
        eprintln!("Dropped");
    }
}

fn init_b(s: &uninit Box<u32>) -> Initialised<'_> {
    *s = Default::default();
}

let s: Struct;
s.field1 = init_b(_);
s.field2 = init_b(_);
// MISTAKE: s is never notarised and thus "Dropped" will not be printed.
```

## API sketch

* [Rust playground link](https://play.rust-lang.org/?version=nightly&mode=debug&edition=2024&gist=71fc6309242bcb601ec150d7461413c1)
