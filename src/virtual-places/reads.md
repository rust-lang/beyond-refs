# Reading from places

Reading from places is similar to writing (one could say they are dual). There
are a couple of differences:
- Places can be read in several different contexts, whereas there is only one
  way to write to them.
- Reading is closely related to moving out of a place. Any type that does not
  implement `Copy` has to be moved instead of copied. Smart pointers must
  explicitly opt in to allowing this by implementing the `PlaceMove` trait.
- Implementing `PlaceMove` has an additional supertrait: `DropHusk`, we will
  cover it in the section about [dropping places](./dropping.md). Essentially
  `DropHusk` is used for dropping a smart pointer where every field has been
  moved out.

The place operation of reading is encoded by the following trait:

```rust
pub unsafe trait PlaceRead<P>: HasPlace
where
    P: Projection<Source = Self::Target>,
    P::Target: Sized,
{
    unsafe fn read(this: *const Self, proj: P) -> P::Target;
}
```

And opting into moving out of a smart pointer is handled by this trait:

```rust
pub unsafe trait PlaceMove<P>: PlaceRead<P> + DropHusk
where
    P: Projection<Source = Self::Target>,
    P::Target: Sized,
{}
```

Not implementing this trait will result in an error when trying to read the
subplace represented by `P` where `P::Target` does not implement the `Copy`
trait.

## Desugaring

A read operation looks like this:

TODO: add all the contexts in which it can appear

It is desugared to:

```rust
<op_type_of_place!(place) as PlaceRead<op_proj_type_of!(place)>>::read(
    op_ptr_to_place!(place),
    op_proj_value_of!(place),
)
```

`PlaceMove` does not play any role in desugaring and is handled by compiler
magic.


## Running example

```rust
unsafe impl<T: ?Sized, P> PlaceRead<P> for MyBox<T>
where
    P: Projection<Source = T>,
    P::Target: Sized,
{
    unsafe fn write(this: *const Self, proj: P) -> P::Target {
        let ptr: *mut T = unsafe { (*this).0 }.as_ptr();
        let ptr: *mut P::Target = unsafe { proj.project_ptr(ptr) };
        unsafe { ptr::read(ptr) }
    }
}

// We'll provide the `DropHusk` impl later
unsafe impl<T: ?Sized, P> PlaceMove<P> for MyBox<T>
where
    P: Projection<Source = T>,
    P::Target: Sized,
{}
```

What our example looks like so far:

```rust
unsafe impl<T: ?Sized, P> PlaceWrite<P> for MyBox<T>
where
    P: Projection<Source = T>,
    P::Target: Sized,
{
    unsafe fn write(this: *const Self, proj: P, value: P::Target) {
        let ptr: *mut T = unsafe { (*this).0 }.as_ptr();
        let ptr: *mut P::Target = unsafe { proj.project_ptr(ptr) };
        unsafe { ptr::write(ptr, value) };
    }
}

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

impl<T: ?Sized> HasPlace for MyBox<T> {
    type Target = T;
}

pub struct MyBox<T: ?Sized>(NonNull<T>);
```
