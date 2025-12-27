# Receiver

{{#include stub.md}}

The Receiver trait enables "arbitrary self types" by doing two things:

* Defining when a smart pointer type is allowed to be a method receiver.
* Generalizing method receivers past types that implement `Deref`.

## Resources

[3519-arbitrary-self-types-v2 - The Rust RFC Book](https://rust-lang.github.io/rfcs//3519-arbitrary-self-types-v2.html)

* [Tracking issue for RFC 3519: `arbitrary_self_types` · Issue #44874 · rust-lang/rust](https://github.com/rust-lang/rust/issues/44874)
* [Arbitrary self types v2: stabilize by adetaylor · Pull Request #135881 · rust-lang/rust](https://github.com/rust-lang/rust/pull/135881)
