# Borrow checker integration

{{#include ../stub.md}}

Due to our fundamental changes to how places are operated upon, we also need to
change how the borrow checker works. It needs to check all of our custom
operations exactly as it checks the current ones. In addition, we need to
provide an API by which implementers of the place operations can decide what
kind of behavior their custom types need from the borrow checker. For example,
`Cell<T>` might allow direct writes `*cell = value;`, those do not require
exclusive access, since `Cell::set` also does not.

## Borrow checker concepts

There are three separate concepts that we are going to introduce to specify the
borrow checker behavior for current references. It will turn out to be
sufficient for all of the other types that we are currently trying to support.

### State of a place

The borrow checker tracks the initialization state of places. When moving out
of a place, this state is changed to `Uninitialized`. It also dictates whether
implicit drops need to be generated or if a drop flag is needed. So the
available states of a place are:

```rust
#[non_exhaustive]
pub enum PlaceState {
    Initialized,
    InitializedPinned,
    Uninitialized,
}
```

The `InitializedPinned` state might go away, depending on the outcome of the
`Move` trait effort (TODO: link to project goal when available).

This state is tracked by the borrow checker locally in each function. At the
function boundaries, a known valid state is required for each place (how does
this translate to raw pointers?).

Since pointers are allowed to handle multiple states (for example, raw pointers
do not place any restrictions on the place's state), we also require an enum to
encode all subsets of states used today:

```rust
#[non_exhaustive]
pub enum PlaceStateSet {
    Any,
    Initialized,
    InitializedAndPinned,
    InitializedAndNotPinned,
    Uninitialized,
}
```

Since a place operation can change the state of a place (for example moving
out, or writing to a moved-out place), we also need *actions* that can be
performed on a place to change its state:

```rust
pub enum PlaceAction {
    Nothing,
    Initialize,
    Overwrite,
    Uninitialize,
    Pin,
    PinInitialize,
}

impl PlaceAction {
    pub fn perform(self, state: PlaceState) -> PlaceState {
        match (self, state) {
            (Self::Nothing, state) => state,

            (Self::Initialize, PlaceState::Uninitialized) => PlaceState::Initialized,
            (Self::Initialize, initialized) => initialized,

            (Self::Overwrite, _) => PlaceState::Initialized,

            (Self::Uninitialize, _) => PlaceState::Uninitialized,

            (Self::Pin, PlaceState::Initialized | PlaceState::InitializedPinned)
                => PlaceState::InitializedPinned,
            (Self::Pin, PlaceState::Uninitialized) => PlaceState::Uninitialized,

            (Self::PinInitialize, _) => PlaceState::InitializedPinned,
        }
    }
}
```

### Access kind

In addition to the state of each place, the borrow checker tracks which *kind*
of access pointers (in our proposal types that implement `HasPlace`) require to
their place. `&T` requires shared access, while `&mut T` needs exclusivity. Raw
pointers don't require any kind of access, as they are *untracked* by the
compiler. Expressed as an enum, we have the following:

```rust
#[non_exhaustive]
pub enum PlaceAccess {
    Shared,
    Exclusive,
    Untracked,
}
```

### Timing of the access

Whenever a place operation is performed, the borrow checker also keeps track of
how long that operation lasts and how long it requires the corresponding
access. An operation can be instant, indefinite, or last for the duration of a
lifetime. Since the last two do not have a lifetime, we cannot use an enum to
describe this concept. Instead we use a trait:

```rust
#[sealed]
pub trait Timing {}

// Types implementing `Timing`:
pub struct Instant;
pub struct Indefinite;
pub struct CovariantLifetime<'a>(PhantomData<&'a ()>);
pub struct ContravariantLifetime<'a>(PhantomData<fn() -> &'a ()>);
pub struct InvariantLifetime<'a>(PhantomData<fn(&'a ()) -> &'a ()>);
```

The additional benefit of this approach is that we can encode the variance of
the lifetime directly in the timing.

## Integrating the place operations

Each place operation will declare the following associated items in addition to
their existing ones:

```rust
    /// The set of states allowed to perform this operation
    const STATE: PlaceStateSet;

    /// The kind of access required to perform this operation
    const ACCESS: PlaceAccess;

    /// The duration of the access required to perform the operation
    type Timing: Timing;

    /// When the operation is finished (after `Timing` expires), the state of
    /// the place is changed according to this action.
    const ACTION: PlaceAction;
```

- `PlaceDrop` has a fixed action (`Uninitialize`) and timing (`Instant`), but
  allows customizing the access and state set (it requires it to be a superset
  of `Initialized`).
- `PlaceDeref` does not declare any of these items. It always participates in
  place operations together with another operation that does declare them, so
  those values are used.
- `PlaceWrite` also declares a `const ALLOW_PRIOR_DROP: bool`, which makes the
  borrow checker insert a `PlaceDrop` before the write in case the expected
  state for writing is `Uninitialized`, but the place written to is
  initialized.

## Examples

### `Box<T>`

We use `@<Type>` to abbreviate `PlaceBorrow<_, Type>` in order to prevent a
scrollbar in the table. We similarly abbreviate `CovariantLifetime<'a>` with
`CovLt<'a>`.

| Operation | State | Access | Timing | Action |
| --------- | ----- | ------ | ------ | ------ |
| `PlaceRead` | `Initialized` | `Shared` | `Instant` | `Nothing` |
| `PlaceWrite` | `Uninitialized` | `Exclusive` | `Instant` | `Initialize` |
| `@<&'a T>` | `Initialized` | `Shared` | `CovLt<'a>` | `Nothing` |
| `@<&'a mut T>` | `Initialized` | `Exclusive` | `CovLt<'a>` | `Nothing` |
| `@<*const T>` | `Any` | `Untracked` | `Indefinite` | `Nothing` |
| `@<*mut T>` | `Any` | `Untracked` | `Indefinite` | `Nothing` |

(note that when we write `T` in the table, in the actual impl, it will be
`P::Target` with `P::Source = T` for any projection `P`)


### `&'a T`

| Operation | State | Access | Timing | Action |
| --------- | ----- | ------ | ------ | ------ |
| `PlaceRead` | `Initialized` | `Shared` | `Instant` | `Nothing` |
| `PlaceWrite` | `Uninitialized` | `Exclusive` | `Instant` | `Initialize` |
| `@<&'b T>` | `Initialized` | `Shared` | `CovLt<'b>` | `Nothing` |
| `@<&'b mut T>` | `Initialized` | `Exclusive` | `CovLt<'b>` | `Nothing` |
| `@<*const T>` | `Any` | `Untracked` | `Indefinite` | `Nothing` |
| `@<*mut T>` | `Any` | `Untracked` | `Indefinite` | `Nothing` |

### `Arc<T>`

| Operation | State | Access | Timing | Action |
| --------- | ----- | ------ | ------ | ------ |
| `PlaceRead` | `Initialized` | `Shared` | `Instant` | `Nothing` |
| `@<&'a T>` | `Initialized` | `Shared` | `CovLt<'a>` | `Nothing` |
| `@<*const T>` | `Any` | `Untracked` | `Indefinite` | `Nothing` |
| `@<*mut T>` | `Any` | `Untracked` | `Indefinite` | `Nothing` |
| `@<ArcRef<T>>` | `Initialized` | `Shared` | `Indefinite` | `Nothing` |

### `*const T`

All of these are `unsafe`:

| Operation | State | Access | Timing | Action |
| --------- | ----- | ------ | ------ | ------ |
| `PlaceRead` | `Any` | `Untracked` | `Instant` | `Nothing` |
| `PlaceWrite` | `Any` | `Untracked` | `Instant` | `Nothing` |
| `@<&'a T>` | `Any` | `Untracked` | `CovLt<'a>` | `Nothing` |
| `@<&'a mut T>` | `Any` | `Untracked` | `CovLt<'a>` | `Nothing` |
| `@<*const T>` | `Any` | `Untracked` | `Indefinite` | `Nothing` |
| `@<*mut T>` | `Any` | `Untracked` | `Indefinite` | `Nothing` |

Thoughts:
- maybe we should record in `HasPlace` if the ptr needs to track the state, since this doesn't
- we could change the behavior of `*raw_ptr = value;` to not drop the original value, but needs an edition.

### `LocalPlace<T>`

| Operation | State | Access | Timing | Action |
| --------- | ----- | ------ | ------ | ------ |
| `@<&'a T>` | `Initialized` | `Shared` | `CovLt<'a>` | `Nothing` |
| `@<&'a mut T>` | `Initialized` | `Exclusive` | `CovLt<'a>` | `Nothing` |
| `@<*const T>` | `Any` | `Untracked` | `Indefinite` | `Nothing` |
| `@<*mut T>` | `Any` | `Untracked` | `Indefinite` | `Nothing` |


## Resources

- [Virtual Places and Borrow Checker Integration | Benno's Blog](https://bennolossin.github.io/blog/field-projections/virtual-places-and-borrowck.html), Dec 2025
