# Exclusive reference collections

In some cases it can be useful to group up multiple exclusive references into a
single collection.

```rust
#[derive(Reborrow)]
struct MutCollection<'a, 'b, 'c> {
    a: &'a mut A,
    b: CustomMut<'b, B>,
    c: Option<&'c mut C>,
}
```

Reborrowing such a collection as exclusive means simply reborrowing each
exclusive reference individually and producing a new collection of the results.
This can also be applied recursively:

```rust
#[derive(Reborrow)]
struct BiggerCollection<'a, 'b, 'c, 'd, 'e, 'f, 'g> {
    one: MutCollection<'a, 'b, 'c>,
    two: MutCollection<'d, 'e, 'f>,
    three: CustomMut<'g, G>,
}
```
