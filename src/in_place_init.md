# In-place initialization

Initializing values in-place without copying them. This eliminates unnecessary copies and allows for self-referential datastructures.

## Range of use cases

TODO: Cover the range of use cases like

* Pinned vs unpinned
* Constructor functions
* Fallible

## Approaches

* [Init expressions](./init_exprs.md)
* [Out pointers](./out_pointers.md)
* [Placing](./placing.md)
* [Guaranteed value emplacement](./guaranteed_emplacement.md)

## Potential design axioms

TODO: Add more or remove.

* Support abstraction through function boundaries

## Resources

[#t-lang/in-place-init](https://rust-lang.zulipchat.com/#narrow/channel/528918-t-lang.2Fin-place-init)
