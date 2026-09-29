# Places (2025-10 until 2026-05)

- [**Rendered Documentation**](https://rust-lang.github.io/beyond-refs/field-projections/legacy/01-places/_01_places/index.html)

- Idea started by [Nadri on Zulip](https://rust-lang.zulipchat.com/#narrow/channel/522311-t-lang.2Fcustom-refs/topic/Field.20projections.20and.20places/near/545831862)
- After lot's of discussion, Nadri created a blog post distilling the idea: <https://nadrieril.github.io/blog/2025/11/11/truly-first-class-custom-smart-pointers.html>

## Core Ideas

- Extend the existing notion of *places* in Rust.
- Make each place operation available via a trait just like `Add` does for `+`.

This results in a very natural feature extending the capabilities of Rust in an incredibly idiomatic way:
- the syntax has almost no changes compared to existing operations on places, except custom borrowing, which uses `@` instead of `&`.
- native support for deeply nested disjoint borrows (which wasn't present in the previous proposal).
