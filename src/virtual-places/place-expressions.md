# Complete place expressions

We already covered the [basic place expressions](./basic-place-expressions.md)
earlier. Now we add place wrappers into the mix

## What is a *place*?

TODO:

    /// Wrapping a place, `k#wrap p`.
    Wrap(Box<PlaceExpr>),

## Desugaring

TODO: update algo

## Converting to projections

Place expressions are closely linked to projections. When performing a place
operation on a place expression, we need to supply the operation with a pointer
to the place where the operation takes place as well as with a value of the
projection. To simplify the work in the desugaring, we introduce two macros
that turn a place expression into a type and value for the outermost
projection.

- The first macro is called `op_proj_type_of!(place)` it is defined recursively
  on place expressions:
  - if `place == local`, then it expands to
    ```rust
    <LocalPlace<typeof(local)> as PlaceWrapper<
        EmptyProjection<typeof(local)>,
    >>::WrappedProjection
    ```
  - if `place == static`, then it expands to
    ```rust
    <StaticPlace<typeof(static)> as PlaceWrapper<
        EmptyProjection<typeof(static)>,
    >>::WrappedProjection
    ```
  - if `place == *q`, then it expands to
    ```rust
    EmptyProjection<typeof(*q)>
    ```
  - if `place == q.field`, then it expands to
    ```rust
    ComposedProjection<
        op_proj_type_of!(q),
        FieldProjection<typeof(q), field_of!(typeof(q), field)>,
    >
    ```
  - if `place == q[idx]`, then it expands to
    ```rust
    ComposedProjection<op_proj_type_of!(q), IndexProjection<typeof(q)>>
    ```
  - if `place == k#wrap (*q).proj`, then it expands to
    ```rust
    <typeof(q) as PlaceWrapper<op_proj_type_of!((*q).proj)>>::WrappedProjection
    ```

- The first macro is called `op_proj_value_of!(place)` it is also defined
  recursively on place expressions:
  - if `place == local`, then it expands to
    ```rust
    <LocalPlace<typeof(local)> as PlaceWrapper<
        EmptyProjection<typeof(local)>,
    >>::wrap_projection(EmptyProjection::default())
    ```
  - if `place == static`, then it expands to
    ```rust
    <StaticPlace<typeof(static)> as PlaceWrapper<
        EmptyProjection<typeof(static)>,
    >>::wrap_projection(EmptyProjection::default())
    ```
  - if `place == *q`, then it expands to 
    ```rust
    EmptyProjection::default()
    ```
  - if `place == q.field`, then it expands to
    ```rust
    ComposedProjection(op_proj_value_of!(q), FieldProjection::default())
    ```
  - if `place == q[idx]`, then it expands to
    ```rust
    ComposedProjection(op_proj_value_of!(q), IndexProjection::new(idx))
    ```
  - if `place == k#wrap (*q).proj`, then it expands to
    ```rust
    <typeof(q) as PlaceWrapper<
        op_proj_type!((*q).proj),
    >>::wrap_projection(op_proj_value_of!((*q).proj))
    ```

## Interactive desugaring algorithm

- TODO: add an interactive web interface for using the compute_place_ty crate
