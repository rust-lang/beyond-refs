# Out pointers `&uninit`

Out pointer intends to model the missing reference categories that capture the **initialisation**
state and the **drop obligation** of a place behind a borrow.

These new reference types promise to provide maximal flexibility around control flow constructs
and effects for the in-place initialisation scenarios.

## Relaxation on user variable initialisation

This proposal relaxes the requirement that user variables must be initialised before first access.
Instead, user variables could be written to as part of gradual in-place initialisation.

```rust
struct MyStruct {
    id: u8,
    data: Data,
}
let x: MyStruct;
x.id = 1;
// x is now considered partially initialised; so x.data has no drop obligation, yet.
x.data = make_data();
// x is now considered initialised so that it has drop obligation
```

## Two new reference categories

### Reference type to uninitialised place `&uninit T`

`&uninit T` is a reference to a place of type `T` that remains uninitialised.
The primary purpose of this type is to signal any initialisation analysis that the pointed place is
uninitialised as long as the reference is live.

This is equivalent to `T*` pointer type in C with all its fields filled with uninitialised poison
data.
The only valid access to the place behind the reference is to write or overwrite the constituent
fields, or taking `&uninit` references to the constitutent fields.

When taking `&uninit` reference to a known initialised place, whose derivation does not pass through
either an immutable or mutable reference `&U` or `&mut U`, the place will be dropped in-place and
considered uninitialised.

###### <a id="uninit-ptr-example">Example</a>

```rust
let mut x = MyStruct::default();
let uninit_x = &uninit x;
// there should have been a drop on x, and now x is uninitialised
// so that access to x.id or x.data is now an error
uninit_x.id = 1;
uninit_x.data = make_data();
// now uninit_x is initialised; x is still not but it can be fixed later
// in the next section.
```

### Reference type to `&own<'_> T`

`&own<'_> T` is a reference to a place of type `T` that semantically owns the pointed value.
This reference signals the initialised state of the pointed place and drop obligation as long as the
reference is live.
It allows mutable access via `&mut` reborrow; by dropping an `&own` the value is dropped in place.

When taking `&own` reference to a known initialised place, whose derivation does not pass through
either an immutable or mutable reference `&U` or `&mut U`, the place will be marked uninitialised
and the drop obligation is transferred to the newly generated `&own` value.

```rust
let mut x = MyStruct::default();
let owned_data = &own x.data;
// it is now illegal to access x.data
let _ = owned_data;
// the destructor of `Data` is called on `owned_data`, aka. `x.data`.
x.data = make_data();
// now x is fully initialised again
```

## Notarisation of uninitialised places, making them initialised

In the `&uninit T` section, a place behind an `&uninit` will remain
uninitialised even though the initialisation is actually confirmed through the
`&uninit` reference.
[For example](#uninit-ptr-example), `x` is still considered uninitialised.
The initialisation on `x` can be confirmed by implicit coercion of the `&uninit`
to an `&own<'_>`, given that the place is known to be initialised.
For this a new statement kind `<-` is proposed, which is inspired by
[`Init` proposal](../init-expr.md), to signal initialisation with the `&own`.

```rust
let init_x: &own<'_> MyStruct = uninit_x;
x <- init_x; // Mark x as initialised.
// Now x is initialised
dbg!(x.id); // OK
```

For the following discussion, we will name the lifetimes on types of `uninit_x`
and the corresponding `&own` as `&'a uninit MyStruct` and `&'b own<'c> MyStruct`.
When coercion happens on `uninit_x`, the relationship `'a == 'c` is established.
Later, when `<-` is used, the lifetime for which the `x` remains uninitialised
is matched against the so-called origin lifetime on the `&own` and the statement
is only accepted when the two lifetime matches. This is true in this case
because the origin lifetime `'c` indeed matches the lifetime `'a` on `uninit_x`.

This also makes the initialisation across function boundaries expressible in
Rust as well, as we now have means to carry initialisation state across function
calls using `&own` safely by using the lifetime annotation.
In order for this to work, the proposal demands the `&'b own<'c> MyStruct` type
to have invariance in the `'c` location between the angle bracket.

```rust
fn init_data<'a>(data: &'a uninit Data) -> &'a own<'a> Data;

let x: MyStruct;
x.id = 1;
x.data <- init_data(&uninit x.data);
// x is now fully initialised

// async initialisation is possible
async fn init_data<'a>(data: &'a uninit Data) -> &'a own<'a> Data;
let x: MyStruct;
x.id = 1;
x.data <- init_data(&uninit x.data).await;

// composing with fallibility is possible
async fn async_init_data<'a>(data: &'a uninit Data)
    -> Option<&'a own<'a> Data>;

let x: MyStruct;
x.data <- async_init_data(&uninit x.data).await?;
// ^ on None case, x.data is uninitialised so there is no drop obligation on
// this field

// this composes well with arbitrary workflow
x.data <- loop {
    if let Some(data) = init_data(&uninit x.data) {
        break data;
    }
};
```

There is more discussion on the semantics of `&own` and `&uninit` references in
the linked document, covering topics of drop flags, MIR semantics as well as
syntatical concerns and various extension proposals.

Out pointers are a fundamental building block of in-place initialisation: at the
core level it is hard to imagine any way of building the feature on a concrete
machine without bringing in out pointers.

The viewpoint of the out pointer approach to in-place initialisation is
therefore that Rust should not attempt to hide the existence of out pointers
behind special traits, attributes, or compiler magic but should instead make
them available as a first class citizen of the language.

The basic idea is a pair of move-only reference-like types `Uninit` and `InPlace`:

```rust
struct Uninit<'a, T>(&'a mut MaybeUninit<T>, Invariant<'a>);
struct InPlace<'a, T>(Box<T, InPlaceAlloc<'a>>, Invariant<'a>);

fn init_x<'a>(x: Uninit<'a, X>) -> InPlace<'a, X> { ... }

fn main() {
    // Note: we'd actually need generativity here to produce a truly unique lifetime for x.
    let mut x = MaybeUninit::<X>::uninit();
    let x = init_x(Uninit::from(&mut x));
}
```

## Approaches

* Custom types (as above)
* [`&uninit`](./in-place-init/uninit-ref.md)

## Resources

[In-place initialization via outptrs](https://hackmd.io/awB-GOYJRlua9Cuc0a3G-Q), Jul 8 2025

* Introduced `InPlace<T>` as a `Box` with custom allocator.
* Very influential.

[Thoughts on "out"-pointer](https://hackmd.io/zpPq14e3Qi6GqEc6fFcy1g?view), Nov 12 2025
