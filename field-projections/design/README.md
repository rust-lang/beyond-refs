# Handles (2026-05 until now)

- [**Rendered Documentation**](https://rust-lang.github.io/beyond-refs/field-projections/current/design/index.html)

- Discussion around field-by-field projections [started on github](https://github.com/rust-lang/rust/pull/154940#discussion_r3143910063) by Nadri through Mark's questions
- [Continued by Nadri on Zulip](https://rust-lang.zulipchat.com/#narrow/channel/522311-t-lang.2Fcustom-refs/topic/Using.20places.20for.20intermediate.20subplaces/near/594089728)
- [Combined with the idea of handles by Benno](https://rust-lang.zulipchat.com/#narrow/channel/522311-t-lang.2Fcustom-refs/topic/Using.20places.20for.20intermediate.20subplaces/near/595608641) 

## Core Ideas

- Turn pointers into handles that can deal with temporarily invalid state.
- Don't expose handles to the user.
- Make the borrow checker ensure that only valid operations are performed on handles.

This results in place operation traits that are much simpler:
- No subplace generics, as we now have field-by-field projections (without their previous disadvantages).
- No raw pointers (except one trait) in the trait signatures (under the hood there still are raw pointers of course).
