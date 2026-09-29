# Field Projection Designs

[**Rendered**](https://rust-lang.github.io/beyond-refs/field-projections/)

## Overview

A collection of approaches to designing the field projection feature for Rust.
This collection aims to provide quick comparisons between the approaches and
the ability to write desugared examples that work in Rust today.

The design of these approaches is incredibly dynamic, lots of small changes
accumulate over time, making it difficult to judge when the current approach
has changed to a new one. For this reason, and to avoid calling every tiny
change a "new approach", we chose to only call substantial, design philosophy
changes a *new approach.*

Links to types, files or other elements in on this website are sadly not
intended to be permanent. Instead it's trying to be a dynamically changing
repository showing the state of the most up-to-date idea.

The approaches currently recorded in this collection are:
- [Handles](./design) (the main approach from 2026-05 until now)
- [Places](./legacy/01-places) (the main approach from 2025-10 until 2026-05)

There are several more historical approaches, albeit it's difficult to truly
categorize the number of the variations; many changes have been lost to time
and in the very beginning there were too many substantial changes in very
little time. Instead of trying to explain that history here, we just provide a
link collection to earlier ideas:

- [Field Projection Project Goal | 24-09-2025 until now](https://github.com/rust-lang/rust-project-goals/issues/390)
- [RFC: field projections v2 | 04-12-2024](https://github.com/rust-lang/rfcs/pull/3735)
- [RFC: Field projection (v1) | 21-09-2022](https://github.com/rust-lang/rfcs/pull/3318)
- [[Pre-RFC] `field_projection` | 13-09-2022](https://internals.rust-lang.org/t/pre-rfc-field-projection/17383)
