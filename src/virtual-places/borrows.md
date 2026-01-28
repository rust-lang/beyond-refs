# Borrowing places

{{#include ../cur.md}}

Borrowing a place is the most complicated operation of the three place
operations. This is because we not only support borrowing using built-in
references and raw pointers, but also with any custom type. For this we need
new syntax to express with which type the place is borrowed. At the moment, we
have chosen `@Type place` as a placeholder syntax; it already provides all of
the ergonomic benefits the final syntax should offer.

In the syntax `@Type place`, `place` is a [place
expression](./place-expressions.md) and `Type` is an identifier. We also have
the fully qualified syntax written as `@<Type> place` where this time, `Type`
can be a type and not only an identifier. Lastly, we have canonical borrows
`@place`, which is the reason for only allowing a single identifier in the
first case. Canonical borrows will be handled last, as they are just syntactic
sugar.

The trait behind `@Type place` takes not only a projection, but also a generic
dictating what `Type` is:

```rust
/// Borrow a place
/// 
/// `X` is the output type of the borrow. The syntax is `@Type place`, and
/// `Type` is substituted for `X`.
pub unsafe trait PlaceBorrow<P, X>: HasPlace
where
    P: Projection<Source = Self::Target>,
{
    unsafe fn borrow(this: *const Self, proj: P) -> X;
}
```

> [!NOTE]
> In addition to the syntax given above, we make `@ref` and `@mut` mean the
> same as `&` and `&mut`. Similarly, we reserve `@raw` and `@raw mut` for
> `&raw const` and `&raw mut` respectively.

## Desugaring

`@Type place` is desugared as follows:

- If `place == local.proj`, then we desugar it to:
  ```rust
  <
      LocalPlace<typeof(local)>
      as
      PlaceBorrow<proj, Type<...>>
  >::borrow(&raw const @%LocalPlace local)
  ```
- If `place == (*place').proj`, then we desugar it to:
  ```rust
  <
      type_of_place!(place')
      as
      PlaceBorrow<proj, Type<...>>
  >::borrow(ptr_to_place!(place'))
  ```

We write `Type<...>` to mean "instantiate `Type` with all generics being
inference variables". When the compiler cannot solve them, they produce an
error as usual and a suggestion to use `@<Type<XYZ>> place` is given, where
`XYZ` are the generics that were resolvable (with `_` as for the unresolved
ones).

The `type_of_place!` and `ptr_to_place!` macros have been introduced in the
section on [dereferencing places](./dereferencing-places.md). `proj` desugars
as described in the section on [projections](./place-projections.md).

### Examples

Here are some examples of how the desugaring of borrows work. Each example uses
fully desugared [place expressions](./place-expressions.md), as that step is
required before attempting this desugaring.

In our examples, we will use the following structs:

```rust
struct Biggest<'a> {
    big: Big<'a>,
}

struct Big<'a> {
    small: Small<'a>,
}

struct Small<'a> {
    smallest: &'a mut Smallest,
}

struct Smallest {
    leaf: Arc<Leaf>,
}

struct Leaf {
    a: usize,
    b: usize,
}
```

#### Basic reborrow

Starting with a very simple example, this is also how normal borrows "should be
desugared" (in practice we might want to special case `&` and `&mut` for perf):

```rust
let x: &'2 mut Biggest<'1>;

let _: &'3 mut Big<'1> = @mut (*x).big;
// desugars to:
let _ = <&'2 mut Biggest<'1> as PlaceBorrow<
    projection!(Biggest<'1>.big),
    &'_ mut _,
>>::borrow(&raw const x);
```

#### Stack borrow

When `x` is already a local, we don't have an implicit dereference. The
desugaring uses `LocalPlace` instead:

```rust
let x: Biggest<'1>;

let _: &'2 mut Small<'1> = @mut x.big.small;
// desugars to:
let _ = <LocalPlace<Biggest<'1>> as PlaceBorrow<
    projection!(Biggest<'1>.big.small),
    &'_ mut _,
>>::borrow(&raw const x);
```

Note how it's only a single call to borrow, with a chained projection of the
two fields.

#### Double deref

When more than one dereference is involved, the `ptr_to_place!` macro uses the
`PlaceDeref` operation in the expansion:

```rust
let x: &'2 mut Biggest<'1>;

let _: &'3 Smallest = @ref *((*x).big.small.smallest);
// desugars to:
let _ = <&'1 mut Smallest as PlaceBorrow<
    projection!(Smallest),
    // ^ this projection is empty, since after the outer `*` above, there is no
    // projection.
    &'_ _,
>>::borrow(
    <LocalPlace<&'2 mut Biggest<'1>> as PlaceDeref<
        projection!(Biggest<'1>.big.small.smallest),
    >>::deref(&raw const x),
);
```

The second projection is only given to the operation, while the inner part is
exclusively passed to `PlaceDeref`:

```rust
let x: &'2 mut Biggest<'1>;

let _: &'3 Arc<Leaf> = @ref (*((*x).big.small.smallest)).leaf;
// desugars to:
let _ = <&'1 mut Smallest as PlaceBorrow<
    projection!(Smallest.leaf),
    // ^ this projection only contains `leaf`, since the rest is hidden behind
    // the inner dereference.
    &'_ _,
>>::borrow(
    <LocalPlace<&'2 mut Biggest<'1>> as PlaceDeref<
        projection!(Biggest<'1>.big.small.smallest),
    >>::deref(&raw const x),
);
```

#### Independence of desugaring

In this example, we illustrate an important property of this desugaring. It
does not fail early when something clearly impossible is written. Every
desugaring that we depend on also still works in this case, however the
compiler will error later when trait solving, as the required `PlaceBorrow` is
not implemented.

```rust
let x: &'2 mut Biggest<'1>;

let _: ArcRef<Small<'1>> = @ArcRef (*x).big.small;
// desugars to:
let _ = <&'2 mut Biggest<'1> as PlaceBorrow<
    projection!(Biggest.big.small),
    ArcRef<_>,
>>::borrow(&raw const x);
```

#### Deep custom borrow

Lastly, we give one complicated example where we borrow the `leaf` field using
the `ArcRef` type:

```rust
let x: &'2 Biggest<'1>;

let _: ArcRef<usize> = @ArcRef (*(*((*x).big.small.smallest)).leaf).a;
// desugars to:
let _ = <&'2 Biggest<'1> as PlaceBorrow<
    projection!(Leaf.a),
    ArcRef<_>,
>>::borrow(
    <&'1 mut Struct<'1> as PlaceDeref<
        projection!(Smallest.leaf),
    >>::deref(
        <LocalPlace<&'2 Struct<'1>> as PlaceDeref<
            projection!(Biggest<'1>.big.small.smallest),
        >>::deref(&raw const x),
    )
);
```
