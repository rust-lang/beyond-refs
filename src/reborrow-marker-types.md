# Marker types

Sometimes users want to define custom types with exclusive reference semantics
that do not contain any pointers at all. This is useful in encoding exclusive
access to data in the function's indirect context. For example, embedded systems
sometimes use reborrowable ZST marker types to pass exclusive access to hardware
peripherals through their call stacks. The author uses a marker ZST to model
garbage collection safepoints, ensuring that unrooted custom references to
garbage collectable data in a system with a moving GC are not held past
safepoints.

```rust
struct CustomMarker<'a>(PhantomData<&'a mut ()>);
```
