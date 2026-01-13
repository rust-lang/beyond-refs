# Autoref

{{#include stub.md}}

Autoref is the technical term used to describe the insertion of automatic borrowing of variables to calling methods. For example:

```rust
struct MyStruct;

impl MyStruct {
    fn method(&self) {}
}

fn main() {
    let my_struct: MyStruct = MyStruct;
    my_struct.method();
    // this works, because it is desugared into:
    MyStruct::methods(&my_struct);
}
```

Going beyond references means adding support for autoref to custom types. The [`HasPlace` proposal](./has-place.md) provides a way to borrow places, which we will explicitly use in this section. However, using that specific mechanism for borrowing is not required; autoref can work with other approaches as well.

## `Receiver::Target` and `HasPlace::Target`

The `HasPlace` trait has an associated type called `Target`, which is the type of the place when dereferencing `Self`. A priori it is a different type from the associated type `Target` on the `Receiver` trait, which is responsible for allowing a type in the receiver position of a method. We have not yet settled the question on the relationship between the two `Target` types. The options are:
1. Merge the `HasPlace` and `Receiver` traits.
2. Make `Receiver` a supertrait of `HasPlace`.
3. Make `HasPlace` a supertrait of `Receiver`.
4. Keep them separate.

Options 1 and 2 are not a good idea, because implementing `Receiver` prevents a type from introducing inherent methods without breaking downstream users. For this reason we only consider options 3 and 4.

Option 3 could result in error messages that are confusing, since implementing `HasPlace` makes `*p` a valid expression (for `p: Self`  ). However, any operation on `*p` (such as reading, writing, and borrowing) require additional traits to be implemented. If none are implemented, it could be strange to allow `*p` in the first place.

Option 4 has the disadvantage of making the model more complex; there are two `Target` types that one has to keep track of when a type implements both differently. Unless we discover a use-case for the diverging types, we will probably choose option 3.

### A note on `Deref`

The discussions surrounding `Receiver` also mention `Deref` and there was a plan to add a supertrait relationship `Deref: Receiver`. `HasPlace` essentially supersedes `Deref`, which therefore takes it out of this question. We would like to make `Deref: HasPlace`, but that depends on the exact shape of `HasPlace` and the interaction with `DerefMut`.

## Algorithm

The algorithm gets invoked on all method calls. They are generally of the shape `p.method()` where `p` is a place expression. The method call can of course also have arguments, but they are ignored in the algorithm.

We first constructs a list of candidate types. This depends on whether the `Target` types of `HasPlace` and `Receiver` are unified or not.
1. If they are unified, we compute the list `L := [T, T::Target, T::Target::Target, ...]`. The computation of this list is described by the following code snippet:
    ```rust
    iter::successors(Some(T), |ty| {
        if ty.implements_has_place() {
            Some(ty.has_place_target())
        } else {
            None
        }
    })
    ```
2. If they are separate, we compute the list
    ```text
    L := flatten [
        [
            T,
            <T as Receiver>::Target,
            <<T as Receiver>::Target as Receiver>::Target,
            ...
        ],
        [
            <T as HasPlace>::Target,
            <<T as HasPlace>::Target as Receiver>::Target,
            <<<T as HasPlace>::Target as Receiver>::Target as Receiver>::Target,
            ...
        ],
        [
            <<T as HasPlace>::Target as HasPlace>::Target,
            <<<T as HasPlace>::Target as HasPlace>::Target as Receiver>::Target,
            <<<<T as HasPlace>::Target as HasPlace>::Target as Receiver>::Target as Receiver>::Target,
            ...
        ],
        ...
    ]
    ```
    The computation of this list is described by this code snippet:
    ```rust
    iter::successors(Some(T), |ty| {
        if ty.implements_has_place() {
            Some(ty.has_place_target())
        } else {
            None
        }
    })
    .flat_map(|ty| iter::successors(Some(ty), |ty| {
        if ty.implements_receiver() {
            Some(ty.receiver_target())
        } else {
            None
        }
    }))
    ```

The second step in the algorithm is to iterate over the list of candidate types. Let `U` be the type that we are considering. We look through all impl blocks of the shape `impl U` and `impl Trait for U` (including generic ones such as `impl<X> Trait for X` where `X` can be substituted by `U`). This gives us a set of *method candidates*. If there is an inherent method, we pick that and continue wit the next step. If there are multiple trait methods, we fail with an ambiguity error. If there are none, we proceed with the next element in the type candidate list.

The third step inspects the method, which has a general shape of `fn method(self: X)` again with function arguments omitted. Now we inspect `X`:
- If `X` occurs in the candidate list that we walked to arrive at this method, we let `q := *...*p` be suitably derefed to get to `X`, which is the number of `HasPlace::Target` we go through. We then desugar the method to `U::method(q)` or `<U as Trait>::method(q)`.
- If `X` does not occur in the already considered candidates then `X: HasPlace` must be true. If that's not the case, we emit an error.
  - If `X::Target` occurs in the already considered candidate, we then let `q := *...*p` be suitably derefed to get to `X::Target`. We then desugar to `U::method(@X q)` or `<U as Trait>::method(@X q)`.
  - If `X::Target` does not occur in the list of already considered candidates, then we continue with the next type from the candidate list.

Note that an alternative that we should consider is to error in the last case.

## Examples

TODO

## Resources

[Autoref and Autoderef for First-Class Smart Pointers | Nadri’s musings](https://nadrieril.github.io/blog/2025/12/18/autoref-and-autoderef-for-first-class-smart-pointers.html), Dec 2025

[Ensure `arbitrary_self_types` method resolution is forward-compatible with custom autoref · Issue #136987](https://github.com/rust-lang/rust/issues/136987#issuecomment-2658112604), Feb 2025
- [issue comment with design sketch](https://github.com/rust-lang/rust/issues/136987#issuecomment-2658112604) (see "Summary sketch")
