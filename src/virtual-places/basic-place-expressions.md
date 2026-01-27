# Basic place expressions

> [!IMPORTANT]
> This section only gives an incomplete explanation of place expressions,
> because giving the full picture requires knowledge of several advanced
> concepts that depend on place expressions. To break the circular dependency,
> we cover a simplified version of place expressions now and revisit them in a
> [later section](./place-expressions.md).
<!-- Make linkcheck happy -->
[!IMPORTANT]: http://example.com

## What is a *place*?

Places in Rust are currently described in the [reference][rust-ref-places] as
expressions that represent a memory location. This of course changes with our
proposal, place expressions can also refer to virtual places, which do not
reference a location in memory. Nevertheless, the nature of place expressions
does not change.

[rust-ref-places]: https://doc.rust-lang.org/nightly/reference/expressions.html#place-expressions-and-value-expressions

Place expressions are used in what the reference calls *place expression
contexts*. These uses are mapped to the various place operations which we cover
later. For this section we are only considering the place expression part. For
example the `(*p).field` subexpression in `let field = &(*p).field;`.

To make it painfully obvious what a place expression is, here is a Rust enum
representing all possible forms a place expression could take:

```rust
/// A place expression.
///
/// A place expression is a recursive data structure. The leaf nodes are
/// local variables (any local variable is a place expression) and static
/// variables. For the recursive cases, given a place expression `p`, we can:
/// - dereference it, written as `*p`,
/// - access a field, written as `p.field`, where `field` is an identifier,
/// - index into it, written as `p[expr]`, where `expr` is any expression.
#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
pub enum PlaceExpr {
    /// Variable `v`.
    Var(Var),
    /// Dereferencing a place `*p`.
    Deref(Box<PlaceExpr>),
    /// Accessing a field `p.field`, `field` can be any identifier.
    FieldAccess(Box<PlaceExpr>, String),
    /// Indexing a place `p[42]`, the index can be an arbitrary expression.
    Index(Box<PlaceExpr>, Expr),
}

pub enum Var {
    /// Local variable `v`.
    Local(Local),
    /// Static variable `v`.
    Static(Static),
}
```

We will later extend the `PlaceExpr` enum with [place
wrappers](./place-wrappers.md), after explaining place operations.

## Desugaring

When writing a place expression, certain dereference operations can be left
implicit and will be inferred by the compiler. Turning a place expression with
implicit dereferences into a place expressions with no implicit dereferences is
called *desugaring the place expression*. During this process, we also obtain
the type of the place expression. This will be important for the place
operations--- for example, writing to a place with `place = value;` computes
the type of `place` and then requires `value` to be of that type.

Desugaring is a simple algorithm: let `p` be the place expression we ought to
desugar. Match on `p`:
- if `p == PlaceExpr::Var(v)`, then we don't need to change anything; the type
  of this place expression is the type of the variable.
- if `p == PlaceExpr::Deref(q)`, then we desugar `q` and compute its type. We
  then require that type to implement the `HasPlace` trait. The type of `p` is
  the `Target` of that type.
- if `p == PlaceExpr::FieldAccess(q, field)`, then we desugar `q` and compute its type.
    - If it has a field of name `field`, then `p` has the type of that field.
    - Otherwise we require the type of `q` to implement `HasPlace` and we add a
      deref to it in `p`. We then check again if it has a field of name `field`
      and iterate until we error or find a field.
- if `p == PlaceExpr::Index(q, expr)`, then we do the same as with
  `FieldAccess`, but instead of the type of the field, we use the type of the
  element type.

The algorithm in pseudo-Rust:

```rust
fn desugar(p: PlaceExpr) -> Result<(PlaceExpr, Type), Error> {
    match p {
        PlaceExpr::Var(var) => Ok((var, var.ty())),
        PlaceExpr::Deref(q) => {
            // We recurse into `q`:
            let (q, q_ty) = desugar(q)?;
            // Since we deref `q`, we require its type to implement `HasPlace`.
            if let Some(target) = Type::get_has_place_target(q_ty) {
                Ok((PlaceExpr::Deref(q), target))
            } else {
                Err(Error::new(q, "should implement `HasPlace`"))
            }
        }
        // Indexing and field access are treated the same.
        PlaceExpr::Index(..) | PlaceExpr::FieldAccess(..) => {
            let mut current_q = match p {
                PlaceExpr::Index(q, _) => q,
                PlaceExpr::FieldAccess(q, _) => q,
                _ => unreachable!(),
            };
            loop {
                let (q, q_ty) = desugar(current_q)?;
                match p {
                    PlaceExpr::Index(_, expr) => {
                        if let Some(elem_ty) = Type::get_indexing_element_ty(q_ty) {
                            return Ok((PlaceExpr::Index(q, expr), elem_ty));
                        }
                    }
                    PlaceExpr::FieldAccess(_, field) => {
                        if let Some(field_ty) = Type::get_field_ty(q_ty, field) {
                            return Ok((PlaceExpr::FieldAccess(q, field), field_ty));
                        }
                    }
                    _ => unreachable!(),
                }
                if Type::get_has_place_target(q_ty).is_none() {
                    return Err(Error::new(q, "should implement `HasPlace`"));
                }
                current_q = PlaceExpr::Deref(q);
            }
        }
    }
}
```

## Examples

### Basic deref

Given the following:
- `Box<T>: HasPlace<Target = T>`
- `Struct.field: Field`

We desugar `p.field` with `p: Box<Struct>` to:
- `(*p).field: Field`

### Multi deref

Given `Struct.field: Field`, we desugar `p.field` with `p: &&&&Struct` to:
- `(****p).field: Field`

### Nested fields

Given:
- `Struct.nested: Box<Nested>`
- `Nested.field: &Field`
- `Field.last: Last`

We desugar `p.nested.field.last` with `p: &Struct` to:
- `(*(*(*p).nested).field).last`
