# Out pointers

{{#include stub.md}}

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
