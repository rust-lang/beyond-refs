#![feature(ptr_metadata)]
#![allow(non_camel_case_types)]

use _01_places::*;

type Ptr<T> = *mut T;

struct Foo {
    a: A,
    ptr_a: Ptr<A>,
}
// Name the projections after the fields to make the expressions below cute.
mk_field_proj!(struct a(Foo.a: A));
mk_field_proj!(struct ptr_a(Foo.ptr_a: Ptr<A>));

struct A {
    b: B,
}
mk_field_proj!(struct b(A.b: B));

struct B {
    n: u32,
}
mk_field_proj!(struct n(B.n: u32));

#[expect(unused)]
fn simple_project(p: Ptr<Foo>) -> Ptr<A> {
    unsafe { p!(@Ptr (*p).a) }
}

#[expect(unused)]
fn double_deref(p: Ptr<Ptr<Foo>>) -> Ptr<A> {
    unsafe { p!(@Ptr (**p).a) }
}

#[expect(unused)]
fn deref_inside_place(p: Ptr<Foo>) -> Ptr<B> {
    unsafe { p!(@Ptr (*(*p).ptr_a).b) }
}

fn main() {
    let mut foo = Foo {
        a: A { b: B { n: 42 } },
        ptr_a: std::ptr::null_mut(),
    };
    // Self-ref is fun
    foo.ptr_a = &raw mut foo.a;

    unsafe {
        let p: Ptr<Foo> = &raw mut foo;
        assert_eq!(p!((*p).a.b.n), 42); // read from place
        let ptr_n: Ptr<u32> = p!(@Ptr (*p).a.b.n); // borrow place
        assert_eq!(*ptr_n, 42);
        p!((*p).a.b.n = 73); // write to place
        assert_eq!(*ptr_n, 73);
        assert_eq!(p!((*(*p).ptr_a).b.n), 73); // read via other ptr
        assert_eq!(p!(p.*.ptr_a.*.b.n), 73); // postfix deref is fun
    }
}
