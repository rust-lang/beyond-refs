# Dereferencing places

Dereferencing a place is an auxiliary operation that is generated as part of
desugaring the main place operations (reading, writing and borrowing). The
operation itself is encoded by the following trait:

```rust
pub unsafe trait PlaceDeref<P>: HasPlace
where
    P: Projection<Source = Self::Target>,
{
    unsafe fn deref(this: *const Self, proj: P) -> *const P::Target;
}
```

As we have seen in the [place operation overview
section](./place-operations.md), place operations take a raw pointer that
points at the value representing the place. So when implementing this trait for
`Box`, `Self` is `Box<T>` and we dereference that to a subplace of the `T`.

It is worth reiterating the central idea behind dereferences in our approach:

> The outermost dereference in the place expression **is part** of the place
> operation itself. All inner dereferences correspond to the place dereference
> operation.

So the pointer returned by `PlaceDeref::deref` will always be used in another
place operation. So for example when we have `x: Box<Box<T>>` and we do `&**x`,
then the outer operation borrows the box by passing a `*const Box<T>` to the
borrow operation. And the inner operation derefs from `*const Box<Box<T>>` to
`*const Box<T>`.

The projection parameter specifies which subplace we want to dereference, so it
allows us to support `x: Box<(Box<T>, U)>` `&(*(*x).0)`.

## Desugaring dereferences

We are going to define two recursive macros used by the desugaring algorithms
of the main place operations. The first macro will expand to the type of which
we invoke the operation and the other will expand to the pointer where the
operation ought to take place. This second macro will use `PlaceDeref` when we
encounter inner dereference operations.

`op_type_of_place!(place) := op_proj_type_of!(place)::Source`

This is not to be confused with the type of a place expression. This is the
type of the place where the operation takes place.

The macro `op_ptr_to_place!(place)` similarly is defined by matching on `place`:
- if `place == local.proj`, then it expands to `&raw const @%LocalPlace local`
- if `place == static.proj`, then it expands to `&raw const @%StaticPlace static`
- if `place == (*q).proj`, then it expands to:
  ```rust
  <op_type_of_place!(q) as PlaceDeref<
      op_proj_type_of!((*q).proj),
  >>::deref(op_ptr_to_place!(q), op_proj_value_of!((*q).proj))
  ```
- if `place == k#wrap (*q).proj`, then it expands to:
  ```rust
  op_ptr_to_place!(q)
  ```

Then

## Running example

```rust
unsafe impl<T, P> PlaceDeref for MyBox<T>
where
    T: ?Sized,
    P: Projection<Source = T>,
{
    unsafe fn deref(this: *const Self, proj: P) -> *const P::Target {
        let ptr: *mut T = unsafe { (*this).0 }.as_ptr();
        let ptr: *mut P::Target = unsafe { proj.project_ptr(ptr) };
        ptr.cast_const()
    }
}
```

What our example looks like so far:

```rust
unsafe impl<T: ?Sized> PlaceMetadata for MyBox<T> {
    unsafe fn metadata(this: *const Self) -> <Self::Target as Pointee>::Metadata {
        let ptr = unsafe { (*this).0 };
        ptr::metadata(ptr.as_ptr())
    }
}

impl<T: ?Sized> HasPlace for MyBox<T> {
    type Target = T;
}

pub struct MyBox<T: ?Sized>(NonNull<T>);
```
