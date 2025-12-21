# Method-based approach

Today, exclusive references can be implemented in user-land using a method-based
approach:

```rust
trait Reborrow {
    fn reborrow(&mut self) -> Self;
}
```

This captures the most important features of reborrowing: a source instance
`self` has exclusive access asserted on it, and a new `Self` is produced from it
(Some formalisations allow the method to choose its own result type using a
`Self::Target` ADT: this is arguably mixing up reborrowing with a generalised
Deref operation). However, this approach comes with downsides: the method
requires an explicit `&mut self` reference which bounds the resulting `Self`'s
lifetime to the calling function, and the method is user-overridable which leads
to arguably non-idiomatic "spooky action" and a possibility of misuse.

## Bounded lifetimes

When the `fn reborrow` method is called in some outer function `fn outer_fn`,
The outer function must create a `&mut T` reference pointing to the value being
reborrowed:

```rust
fn outer_fn<'a>(t: CustomMut<'a, u32>) -> &'a u32 {
    // This:
    inner_fn(t.reborrow());
    // ... is equivalent to this:
    let t_mut = &mut t;
    inner_fn(t_mut.reborrow())
}
```

This means that the `fn reborrow` method is given a reference pointing to a
local value, effectively a pointer onto the stack. The compiler must make sure
that this pointer does not outlive the stack, which then means that the lifetime
of the resulting `Self` created by `fn reborrow` cannot outlive the function in
which it was created in. In the above example, this means that trying to return
the result of `inner_fn` will not compile because of the `fn reborrow` call,
citing "returns a value referencing data owned by the current function".

Compare this to Rust references: the compiler understands that the result of
reborrowing a `&mut` produces a new reference that can be extended to the
original reference's lifetime. This function compiles despite an explicit
reborrow being performed by the `&mut *t` code.

```rust
fn outer_fn<'a>(t: &'a mut u32) -> &'a u32 {
    inner_fn(&mut *t)
}
```

We can make the code not compile by explicitly creating a `&mut t` reference:

```rust
fn outer_fn<'a>(mut t: &'a mut u32) -> &'a u32 {
    // no longer compiles: returns a value referencing data owned by the current
    // function
    inner_fn(&mut t)
}
```

In user-land code bases that use the explicit `fn reborrow` method, there exists
a way to fix this issue: by simply removing the `fn reborrow` method call, the
original example code will compile. But knowing of this fix requires some deeper
understanding of the `fn reborrow` method and the borrow checker: it is not an
obvious or clean fix.

Most importantly, if the Rust compiler is in charge of automatically injecting
`fn reborrow` calls at appropriate use-sites, then it may not be feasible for
the compiler to perform the analysis to determine if a particular call should be
removed or not. Furthermore, in a post-Polonius borrow checker world it will
become possible for code like this to compile:

```rust
fn inner_fn<'a>(t: &'a mut u32) -> Result<&'a u32, &'a u32> {
    // ...
}

fn outer_fn<'a>(t: &'a mut u32) -> Result<&'a u32, &'a u32> {
    let result = inner_fn(t)?;
    if result > 100 {
        Ok(t)
    } else {
        Err(t)
    }
}
```

This means that a reborrow of `t` must always happen, yet the lifetime expansion
of the result of `inner_fn` depends on whether the function returns `Ok` or
`Err`. In the `Err` branch the result's lifetime must expand to that of the
source `t`, but in the `Ok` branch it must shrink to re-enable usage of the
source `t`. The method-based approach to Reborrow traits will not work here, as
the compiler cannot choose to call the method based on a result value that
depends on the call's result.

One could argue that the compiler could simply extend the lifetime of the method
call's result, as it should only do good deeds. This may well open a soundness
hole that allows safe Rust code to perform use-after-free with stack references.

## User-controlled code

The second downside of the method-based approach is that reborrowing is fully
invisible in the source code, and having user-controlled code appear where there
is none visible is not very idiomatic Rust.

Consider eg. the following code:

```rust
struct Bad;

impl Reborrow for Bad {
    fn reborrow(&mut self) -> Self {
        println!("I'm bad!");
        Self
    }
}

fn main() {
    let bad = Bad;
    let bad = bad;
}
```

Depending on the exact implementation choices, this might print out the "I'm
bad" message. This is especially problematic if the compiler chooses to trust
that `fn reborrow` methods never store their `&mut self` reference given to them
and allows lifetime extension to happen:

```rust
struct Unsound<'a>(&'a mut &'a u32, u32);

// Note: lifetime for trait here to show that the compiler specially trusts that
// these lifetimes indeed are fully unifiable.
impl<'a> Reborrow<'a> for Unsound<'a> {
    fn reborrow(&'a mut self) -> Self {
        let data = &self.1;
        self.0 = data;
        Self(self.0, self.1)
    }
}
```

This would absolutely not compile today, but if the compiler did truly trust
that reborrow methods can do no wrong then something like this might just pass
through the compiler's intentional blindspot and become a soundness hole.
