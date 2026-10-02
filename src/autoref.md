# Autoref and method resolution

Autoref is the technical term used to describe the insertion of automatic borrowing of variables to calling methods. For example:

```rust
struct MyStruct;

impl MyStruct {
    fn method(&self) {}
}

fn main() {
    let my_struct: MyStruct = MyStruct;
    my_struct.method();
    // this works, because it is desugared into:
    MyStruct::methods(&my_struct);
}
```

Going beyond references means adding support for autoref to custom types. The [`HasPlace` proposal](./virtual-places/index.md) provides a way to borrow places. It also describes [an algorithm for autoref and method resolution](./virtual-places/autoref.md) which uses custom borrows. However, using that specific mechanism for autoref and method resolution is not required; autoref can work with other approaches as well.

## Resources

- [Autoref and next gen place traits | Meeting by Xiang, Nadri and Benno](https://hackmd.io/@rust-for-linux-/S1nS-dGHWe), Jan 2026
- [Autoref and Autoderef for First-Class Smart Pointers | Nadri’s musings](https://nadrieril.github.io/blog/2025/12/18/autoref-and-autoderef-for-first-class-smart-pointers.html), Dec 2025
- [Ensure `arbitrary_self_types` method resolution is forward-compatible with custom autoref · Issue #136987](https://github.com/rust-lang/rust/issues/136987#issuecomment-2658112604), Feb 2025
  - [issue comment with design sketch](https://github.com/rust-lang/rust/issues/136987#issuecomment-2658112604) (see "Summary sketch")
