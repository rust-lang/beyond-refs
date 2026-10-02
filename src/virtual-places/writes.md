# Writing to places

The place operation of writing is encoded by the following trait:

```rust
pub unsafe trait PlaceWrite<P>: HasPlace
where
    P: Projection<Source = Self::Target>,
    P::Target: Sized,
{
    unsafe fn write(this: *const Self, proj: P, value: P::Target);
}
```

## Desugaring

A write operation looks like this:

```rust
place = value;
```

It is desugared to:

```rust
<op_type_of_place!(place) as PlaceWrite<op_proj_type_of!(place)>>::write(
    op_ptr_to_place!(place),
    op_proj_value_of!(place),
    value,
);
```


## Running example

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
```

What our example looks like so far:

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

impl<T: ?Sized> HasPlace for MyBox<T> {
    type Target = T;
}

pub struct MyBox<T: ?Sized>(NonNull<T>);
```
