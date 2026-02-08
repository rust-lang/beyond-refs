# Dropping

```rust
pub unsafe trait DropHusk: HasPlace {
    unsafe fn drop_husk(this: *const Self);
}

pub unsafe trait PlaceDrop<P>: DropHusk
where
    P: Projection<Source = Self::Target>,
{
    unsafe fn drop(this: *const Self, proj: P);
}
```

## Running example

```rust
unsafe impl<T: ?Sized> DropHusk for MyBox<T> {
    unsafe fn drop_husk(this: *const Self) {
        let ptr: *const T = unsafe { (*this).0 }.as_ptr();
        let layout = unsafe { Layout::for_value_raw(ptr) };
        unsafe { deallocate(ptr.cast::<u8>(), layout) };
    }
}

unsafe impl<T: ?Sized, P: Projection<Source = T>> PlaceDrop<P> {
    unsafe fn drop(this: *const Self, proj: P) {
        let ptr: *mut T = unsafe { (*this).0 }.as_ptr();
        let ptr: *mut P::Target = unsafe { proj.project_ptr(ptr) };
        unsafe { ptr::drop_in_place(ptr) };
    }
}
```
