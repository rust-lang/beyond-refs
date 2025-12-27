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

[In-place initialization - Rust Project Goals](https://rust-lang.github.io/rust-project-goals/2025h2/in-place-initialization.html), Fall 2025
* [Design meeting 2025-07-30: In-place initialization - HackMD](https://hackmd.io/XXuVXH46T8StJB_y0urnYg)

[#t-lang/in-place-init > in-place initialization: RfL design wishes - rust-lang - Zulip](https://rust-lang.zulipchat.com/#narrow/channel/528918-t-lang.2Fin-place-init/topic/in-place.20initialization.3A.20RfL.20design.20wishes/with/531905430)
