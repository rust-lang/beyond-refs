//! Crate to experiment with the API proposed in
//! <https://nadrieril.github.io/blog/2025/11/11/truly-first-class-custom-smart-pointers.html>.
#![feature(ptr_metadata)]

use std::ptr::NonNull;

mod basic_impls;
mod projection;
pub use projection::*;
mod place_ops;
pub use place_ops::*;

/// Make a unit struct that represents the projection to a particular struct
/// field. Only works for sized types.
///
/// Syntax: `mk_field_proj!(struct FooAProj(Foo.a: A))`.
#[macro_export]
macro_rules! mk_field_proj {
    (struct $name:ident($src_ty:ident.$field:ident: $tgt_ty:ty)) => {
        #[derive(Clone)]
        struct $name;
        impl Projection for $name {
            type Source = $src_ty;
            type Target = $tgt_ty;
            fn offset(
                &self,
                _: <Self::Source as core::ptr::Pointee>::Metadata,
            ) -> usize {
                core::mem::offset_of!($src_ty, $field)
            }
            fn project_metadata(
                &self,
                _: <Self::Source as core::ptr::Pointee>::Metadata,
            ) -> <Self::Target as core::ptr::Pointee>::Metadata {
            }
        }
    };
}

/// Macro that simulates the proposed new syntax. Derefs must be explicit and
/// the identifiers used for field projections must actually be values of some
/// `Projection` type, e.g. built with `mk_field_proj`.
///
/// Examples:
/// ```text
/// (*p).a
/// -> a.read(&raw const p)
/// (*p).a.b
/// -> a.compose(b).read(&raw const p)
/// (**p).a
/// -> a.read(NoopProj::default().deref(&raw const p))
/// (*(*p).ptr_a).b
/// -> b.read(ptr_a.deref(&raw const p))
/// (*p).a = foo()
/// -> a.write(&raw const p, foo())
/// @R *p
/// -> NoopProj::default().borrow::<_, R<_>>(&raw const p)
/// @R (*p).a
/// -> a.borrow::<_, R<_>>(&raw const p)
/// @R (**p).a
/// -> a.borrow::<_, R<_>>(NoopProj::default().deref(&raw const p))
/// ```
#[macro_export]
macro_rules! p {
    // Parse the input syntax. Step one was to check if we're borrowing or not,
    // which happened at the user-facing entrypoint.
    // Step 2: identify the base place, which is either a local or a deref.
    (#parse_base(
        $action:ident($($action_args:tt)*),
        input(
            // A deref of a potentially-complex place expression.
            (*$($place:tt)*)
            $($rest:tt)*
        )
    )) => {{
        use $crate::ProjectionExt;
        let start = p!(#build_start(deref($($place)*)));
        $crate::p!(#parse_proj(
            $action($($action_args)*),
            ptr(start),
            project(),
            input($($rest)*)
        ))
    }};
    (#parse_base(
        $action:ident($($action_args:tt)*),
        input(
            // A deref of a local, with no projections.
            *$local:ident
        )
    )) => {
        $crate::p!(#parse_base(
            $action($($action_args)*),
            input($local.*)
        ))
    };
    (#parse_base(
        $action:ident($($action_args:tt)*),
        input(
            // A postfix deref
            $local:ident.*
            $($rest:tt)*
        )
    )) => {
        $crate::p!(#parse_proj(
            $action($($action_args)*),
            ptr(&raw const $local),
            project(),
            input($($rest)*)
        ))
    };
    (#parse_base(
        $action:ident($($action_args:tt)*),
        input(
            // Not a deref so we remove the parens (to support `(x.a).b`).
            ($($place:tt)*)
            $($rest:tt)*
        )
    )) => {
        $crate::p!(#parse_base(
            $action($($action_args)*),
            input($($place)* $($rest)*)
        ))
    };
    // (#parse_base(
    //     $action:ident($($action_args:tt)*),
    //     input(
    //         // A local
    //         $local:ident
    //         $($rest:tt)*
    //     )
    // )) => {
    //     $crate::p!(#parse_proj(
    //         $action($($action_args)*),
    //         local($local),
    //         project(),
    //         input($($rest)*)
    //     ))
    // };
    // Step 3: gather the possible projections.
    (#parse_proj(
        $action:ident($($action_args:tt)*),
        $start:ident($($start_args:tt)*),
        project($($fields:tt)*),
        input(
            . * // postfix deref
            $($input:tt)*
        )
    )) => {{
        // TODO
        let p = $crate::p!(#parse_proj(
            deref(),
            $start($($start_args)*),
            project($($fields)*),
            input()
        ));
        $crate::p!(#parse_proj(
            $action($($action_args)*),
            ptr(p),
            project(),
            input($($input)*)
        ))
    }};
    (#parse_proj(
        $action:ident($($action_args:tt)*),
        $start:ident($($start_args:tt)*),
        project($($fields:tt)*),
        input(
            .$field:ident
            $($rest:tt)*
        )
    )) => {
        $crate::p!(#parse_proj(
            $action($($action_args)*),
            $start($($start_args)*),
            project($($fields)*.$field),
            input($($rest)*)
        ))
    };
    (#parse_proj(
        $action:ident($($action_args:tt)*),
        $start:ident($($start_args:tt)*),
        project($($fields:tt)*),
        input(
            $(= $rvalue:expr)?
        )
    )) => {
        $crate::p!(#parse_assign(
            $action($($action_args)*),
            $start($($start_args)*),
            project($($fields)*),
            input($(= $rvalue)?)
        ))
    };
    // Step 4: Detect an assignment, if any.
    (#parse_assign(
        read_or_write(),
        $start:ident($($start_args:tt)*),
        project($($proj_args:tt)*),
        input(
            = $rvalue:expr
        )
    )) => {
        $crate::p!(#build(
            write($rvalue),
            $start($($start_args)*),
            project($($proj_args)*),
        ))
    };
    (#parse_assign(
        read_or_write(),
        $start:ident($($start_args:tt)*),
        project($($proj_args:tt)*),
        input()
    )) => {
        $crate::p!(#build(
            read(),
            $start($($start_args)*),
            project($($proj_args)*),
        ))
    };
    (#parse_assign(
        $action:ident($($action_args:tt)*),
        $start:ident($($start_args:tt)*),
        project($($proj_args:tt)*),
        input()
    )) => {
        $crate::p!(#build(
            $action($($action_args)*),
            $start($($start_args)*),
            project($($proj_args)*),
        ))
    };

    // Helpers for the final build.
    // Compose some field projections.
    (#compose_projs()) => { $crate::NoopProj::default() };
    (#compose_projs(.$field:ident $($rest:tt)*)) => {
        $field.compose(p!(#compose_projs($($rest)*)))
    };
    // Build the pointer expression we start with. For non-idents, we call back
    // to our parsing logic to deref a complex place expression.
    (#build_start(deref($ptr:ident))) => { &raw const $ptr };
    (#build_start(deref($($place:tt)*))) => {
        $crate::p!(#parse_base(deref(), input($($place)*)))
    };

    // Evaluate the intermediate values.
    (#build(
        $action:ident($($action_args:tt)*),
        ptr($ptr:expr),
        project($($proj_args:tt)*),
    )) => {{
        use $crate::ProjectionExt;
        let proj = p!(#compose_projs($($proj_args)*));
        $crate::p!(#do_action(
            $action($($action_args)*),
            base($ptr),
            project(proj),
        ))
    }};

    // Now we build the final expression.
    (#do_action(
        read(),
        base($ptr:expr),
        project($proj:expr),
    )) => {
        $proj.read($ptr)
    };
    (#do_action(
        deref(),
        base($ptr:expr),
        project($proj:expr),
    )) => {
        $proj.deref($ptr.cast_mut())
    };
    (#do_action(
        write($rvalue:expr),
        base($ptr:expr),
        project($proj:expr),
    )) => {
        $proj.write($ptr.cast_mut(), $rvalue)
    };
    (#do_action(
        borrow($($ptr_ty:tt)*),
        base($ptr:expr),
        project($proj:expr),
    )) => {
        $proj.borrow::<_, $($ptr_ty)*>($ptr)
    };

    // Catch internal errors instead of looping back to the catch-all case
    // below.
    (#$($rest:tt)*) => {
        compile_error!("Unsupported expression")
    };

    // Entrypoints.
    // @_ place_expr (let inference determine the target pointer)
    (@_ $($place:tt)*) => {
        $crate::p!(#parse_base(borrow(_), input($($place)*)))
    };
    // @Ptr<ty_params> place_expr
    (@$ptr:ident<$($ty:ty),*> $($place:tt)*) => {
        $crate::p!(#parse_base(borrow($ptr<$($ty),*>), input($($place)*)))
    };
    // @Ptr place_expr
    (@$ptr:ident $($place:tt)*) => {
        $crate::p!(#parse_base(borrow($ptr<_>), input($($place)*)))
    };
    // Anything else
    ($($place:tt)*) => {
        $crate::p!(#parse_base(read_or_write(), input($($place)*)))
    };
}
